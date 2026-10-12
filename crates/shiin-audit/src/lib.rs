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
}

/// One record in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Specification version of this event.
    pub schema: String,
    /// Position in the log, starting at 1.
    pub seq: u64,
    /// What happened, such as `decision.made`.
    #[serde(rename = "type")]
    pub event_type: String,
    /// When it happened, as an RFC 3339 timestamp.
    pub time: String,
    /// Hash of the previous event.
    pub prev_hash: String,
    /// Hash of this event's contents and `prev_hash`.
    pub hash: String,
    /// The event's body.
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
    /// The event is written and flushed before this returns, so a decision that
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
        payload: serde_json::Value,
    ) -> Result<AuditEvent, AuditError> {
        let events = self.read()?;
        let (seq, prev_hash) = events.last().map_or((1, GENESIS.to_owned()), |last| {
            (last.seq.saturating_add(1), last.hash.clone())
        });

        let mut event = AuditEvent {
            schema: "shiin.audit-event/v1-draft.1".to_owned(),
            seq,
            event_type: event_type.to_owned(),
            time: time.to_owned(),
            prev_hash,
            hash: String::new(),
            payload,
        };
        event.hash = hash_of(&event);

        let line = serde_json::to_string(&event).unwrap_or_default();
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
        let mut breaks = Vec::new();
        let mut expected_prev = GENESIS.to_owned();
        let mut expected_seq = 1_u64;

        for event in &events {
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
            if hash_of(event) != event.hash {
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
}

/// Computes an event's hash over its contents and the previous hash.
fn hash_of(event: &AuditEvent) -> String {
    let mut hasher = Sha256::new();
    hasher.update(event.schema.as_bytes());
    hasher.update(event.seq.to_string().as_bytes());
    hasher.update(event.event_type.as_bytes());
    hasher.update(event.time.as_bytes());
    hasher.update(event.prev_hash.as_bytes());
    hasher.update(
        serde_json::to_string(&event.payload)
            .unwrap_or_default()
            .as_bytes(),
    );
    format!("sha256:{:x}", hasher.finalize())
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
            serde_json::json!({"id": "a"}),
        )
        .unwrap();
        log.append(
            "decision.made",
            "2026-10-12T10:00:01Z",
            serde_json::json!({"decision": "allow"}),
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
            serde_json::json!({"decision": "deny"}),
        )
        .unwrap();
        log.append(
            "decision.made",
            "2026-10-12T10:00:01Z",
            serde_json::json!({"decision": "deny"}),
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
