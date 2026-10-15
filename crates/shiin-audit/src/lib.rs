//! The audit log: append-only JSON Lines linked by a hash chain.
//!
//! Each event carries the hash of the event before it, so removing or altering an
//! event is detectable ([ADR-0029](../../../docs/adr/0029-hash-chained-jsonl-audit-log.md)).
//! The log is tamper-evident, not tamper-proof: anyone who can write the file can
//! rewrite the whole chain, which is why receipts are signed separately (v0.4).

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shiin_schema::clock;
use thiserror::Error;

/// The genesis value for the first event's `prev_hash`.
pub const GENESIS: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// Why the audit log could not be read or written.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AuditError {
    /// The log file could not be read or written.
    #[error("{path}: {source}")]
    Io {
        /// The file involved.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// A line in the log is not a valid event.
    #[error("{path}: line {line} is not a valid audit event: {source}")]
    Malformed {
        /// The file involved.
        path: PathBuf,
        /// The 1-based line number.
        line: usize,
        /// The underlying error.
        #[source]
        source: serde_json::Error,
    },
    /// A record cannot be encoded without losing audit data.
    #[error("cannot encode audit event: {source}")]
    Encode {
        /// The serialization failure.
        #[from]
        source: serde_json::Error,
    },
    /// An existing log is unsupported or has been altered.
    #[error("{path}: event {seq}: {detail}")]
    Invalid {
        /// The log involved.
        path: PathBuf,
        /// Position at which validation failed.
        seq: u64,
        /// Operator-facing explanation.
        detail: String,
    },
}

impl AuditError {
    /// The stable code for this error, as registered in `docs/reference/error-codes.md`.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match *self {
            Self::Io { .. } | Self::Encode { .. } => "SHIIN-AUD-0001",
            Self::Malformed { .. } | Self::Invalid { .. } => "SHIIN-AUD-0002",
        }
    }
}

/// One record in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEvent {
    /// Specification version of this event.
    pub schema: String,
    /// Unique event identifier (`AUD-005`).
    pub id: String,
    /// Position in the log, starting at 1.
    pub seq: u64,
    /// What happened, such as `decision.made`.
    #[serde(rename = "kind")]
    pub event_type: String,
    /// When it happened, as an RFC 3339 timestamp.
    #[serde(rename = "timestamp")]
    pub time: String,
    /// Hash of the previous event.
    #[serde(rename = "prev")]
    pub prev_hash: String,
    /// Hash of this event's contents and `prev_hash`.
    #[serde(rename = "self")]
    pub hash: String,
    /// An allowlisted summary; command and URL values are replaced with digests.
    pub payload: serde_json::Value,
}

/// Why a chain failed verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainBreak {
    /// Position of the offending event.
    pub seq: u64,
    /// What is wrong.
    pub detail: String,
}

/// An append-only audit log on disk.
pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    /// Opens, or prepares to create, the log at `path`.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The log's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends an event, linking it to the current head.
    ///
    /// The event is written, flushed, and synced before this returns, so a decision that
    /// depends on it being recorded can proceed afterwards
    /// ([ADR-0030](../../../docs/adr/0030-audit-durability.md)).
    ///
    /// # Errors
    ///
    /// Returns an error if the log cannot be read or written.
    pub fn append(
        &self,
        event_type: &str,
        time: &str,
        payload: &serde_json::Value,
    ) -> Result<AuditEvent, AuditError> {
        let id = clock::new_id();
        if id.is_empty() {
            return Err(AuditError::Invalid {
                path: self.path.clone(),
                seq: 0,
                detail: "OS randomness failed; refusing to create an event".to_owned(),
            });
        }
        let events = self.read()?;
        if let Some(problem) = verify_events(&events)?.first() {
            return Err(AuditError::Invalid {
                path: self.path.clone(),
                seq: problem.seq,
                detail: problem.detail.clone(),
            });
        }
        let (seq, prev_hash) = events.last().map_or((1, GENESIS.to_owned()), |last| {
            (last.seq.saturating_add(1), last.hash.clone())
        });

        let mut event = AuditEvent {
            schema: "shiin.audit-event/v1-draft.2".to_owned(),
            id,
            seq,
            event_type: event_type.to_owned(),
            time: time.to_owned(),
            prev_hash,
            hash: String::new(),
            payload: redacted_summary(payload)?,
        };
        event.hash = hash_of(&event)?;

        let line = serde_json::to_string(&event)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|source| AuditError::Io {
                path: self.path.clone(),
                source,
            })?;
        writeln!(file, "{line}").map_err(|source| AuditError::Io {
            path: self.path.clone(),
            source,
        })?;
        file.flush().map_err(|source| AuditError::Io {
            path: self.path.clone(),
            source,
        })?;
        file.sync_all().map_err(|source| AuditError::Io {
            path: self.path.clone(),
            source,
        })?;
        Ok(event)
    }

    /// Reads every event in order. A log that does not exist yet is empty.
    ///
    /// # Errors
    ///
    /// Returns an error if the log cannot be read or holds a malformed line.
    pub fn read(&self) -> Result<Vec<AuditEvent>, AuditError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&self.path).map_err(|source| AuditError::Io {
            path: self.path.clone(),
            source,
        })?;
        let mut events = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(|source| AuditError::Io {
                path: self.path.clone(),
                source,
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let event: AuditEvent =
                serde_json::from_str(&line).map_err(|source| AuditError::Malformed {
                    path: self.path.clone(),
                    line: index.saturating_add(1),
                    source,
                })?;
            events.push(event);
        }
        Ok(events)
    }

    /// Verifies the chain, returning every break found.
    ///
    /// # Errors
    ///
    /// Returns an error if the log cannot be read.
    pub fn verify(&self) -> Result<Vec<ChainBreak>, AuditError> {
        let events = self.read()?;
        verify_events(&events)
    }
}

/// Validates every link, including the schema, before a new event can extend it.
fn verify_events(events: &[AuditEvent]) -> Result<Vec<ChainBreak>, AuditError> {
    let mut breaks = Vec::new();
    let mut expected_prev = GENESIS.to_owned();
    let mut expected_seq = 1_u64;

    for event in events {
        if event.schema != "shiin.audit-event/v1-draft.2" {
            breaks.push(ChainBreak {
                seq: event.seq,
                detail: "unsupported audit schema; preserve the old log separately".to_owned(),
            });
        }
        if event.seq != expected_seq {
            breaks.push(ChainBreak {
                seq: event.seq,
                detail: format!("expected sequence {expected_seq}"),
            });
        }
        if event.prev_hash != expected_prev {
            breaks.push(ChainBreak {
                seq: event.seq,
                detail: "previous hash does not match the event before it".to_owned(),
            });
        }
        if hash_of(event)? != event.hash {
            breaks.push(ChainBreak {
                seq: event.seq,
                detail: "contents do not match the recorded hash".to_owned(),
            });
        }
        expected_prev.clone_from(&event.hash);
        expected_seq = event.seq.saturating_add(1);
    }
    Ok(breaks)
}

/// Hashes the compact JSON object, with lexicographically ordered keys and no `self`.
fn hash_of(event: &AuditEvent) -> Result<String, serde_json::Error> {
    let mut unsigned = serde_json::to_value(event)?;
    if let Some(object) = unsigned.as_object_mut() {
        object.remove("self");
    }
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&unsigned)?)
    ))
}

/// Retains identifiers and paths, never raw commands, URLs, file contents, or host payloads.
fn redacted_summary(payload: &serde_json::Value) -> Result<serde_json::Value, serde_json::Error> {
    use serde_json::{Map, Value};

    let mut summary = Map::new();
    for key in [
        "schema",
        "id",
        "request_id",
        "session",
        "action_type",
        "task",
        "policy_snapshot",
        "decision",
        "reason",
    ] {
        if let Some(value) = payload
            .get(key)
            .filter(|value| value.is_string() || value.is_null())
        {
            summary.insert(key.to_owned(), value.clone());
        }
    }
    for (key, fields) in [
        ("agent", &["id", "assurance"][..]),
        ("workspace", &["root", "vcs"][..]),
        ("host", &["name", "tool"][..]),
    ] {
        if let Some(value) = payload.get(key).and_then(Value::as_object) {
            let selected = fields
                .iter()
                .filter_map(|field| {
                    value
                        .get(*field)
                        .filter(|value| value.is_string())
                        .map(|value| ((*field).to_owned(), value.clone()))
                })
                .collect();
            summary.insert(key.to_owned(), Value::Object(selected));
        }
    }
    if let Some(effects) = payload.get("effects").and_then(Value::as_array) {
        let mut selected = Vec::new();
        for effect in effects {
            let mut entry = Map::new();
            for key in ["kind", "confidence", "action_type"] {
                if let Some(value) = effect.get(key).filter(|value| value.is_string()) {
                    entry.insert(key.to_owned(), value.clone());
                }
            }
            if let Some(value) = effect.get("value").and_then(Value::as_str) {
                if matches!(
                    effect.get("kind").and_then(Value::as_str),
                    Some("path" | "count")
                ) {
                    entry.insert("value".to_owned(), Value::String(value.to_owned()));
                } else {
                    entry.insert(
                        "value_digest".to_owned(),
                        Value::String(format!("sha256:{:x}", Sha256::digest(value.as_bytes()))),
                    );
                }
            }
            selected.push(Value::Object(entry));
        }
        summary.insert("effects".to_owned(), Value::Array(selected));
    }
    summary.insert(
        "digest".to_owned(),
        Value::String(format!(
            "sha256:{:x}",
            Sha256::digest(serde_json::to_vec(payload)?)
        )),
    );
    Ok(Value::Object(summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A log file unique to one test, since tests run in parallel.
    fn temporary_log(name: &str) -> (AuditLog, PathBuf) {
        let mut path = std::env::temp_dir();
        path.push(format!("shiin-audit-{}-{name}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        (AuditLog::new(path.clone()), path)
    }

    #[test]
    fn a_chain_of_appended_events_verifies() {
        let (log, path) = temporary_log("chain");
        log.append(
            "request.received",
            "2026-10-12T10:00:00Z",
            &serde_json::json!({"id": "a"}),
        )
        .unwrap();
        log.append(
            "decision.made",
            "2026-10-12T10:00:01Z",
            &serde_json::json!({"decision": "allow"}),
        )
        .unwrap();

        let events = log.read().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].prev_hash, GENESIS);
        assert_eq!(events[1].prev_hash, events[0].hash);
        assert!(log.verify().unwrap().is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn altering_an_event_breaks_the_chain() {
        let (log, path) = temporary_log("tamper");
        log.append(
            "decision.made",
            "2026-10-12T10:00:00Z",
            &serde_json::json!({"decision": "deny"}),
        )
        .unwrap();
        log.append(
            "decision.made",
            "2026-10-12T10:00:01Z",
            &serde_json::json!({"decision": "deny"}),
        )
        .unwrap();

        // Rewrite the first event's payload, as an attacker covering a denial would.
        let text = std::fs::read_to_string(&path).unwrap();
        let tampered = text.replacen("\"deny\"", "\"allow\"", 1);
        std::fs::write(&path, tampered).unwrap();

        let breaks = log.verify().unwrap();
        assert!(!breaks.is_empty(), "tampering must be detected");
        let _ = std::fs::remove_file(path);
    }
}
