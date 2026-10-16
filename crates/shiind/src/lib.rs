//! Internal Shiin API; it can change in any release.
//!
//! Synchronous, serialized authorization transactions used by the local daemon.

use serde::Deserialize;
use serde_json::{Value, json};
use shiin_approval::{GrantScope, Store, deadline};
use shiin_audit::AuditLog;
use shiin_engine::{Inputs, decide};
use shiin_policy::{CompiledPolicy, toml_front_end};
use shiin_schema::{ActionRequest, Mode, Resolution, TaskDeclaration, TaskUsage, clock};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;

/// Operator credential: generated in memory and never persisted or sent to agent clients.
pub struct OperatorKey(String);

impl OperatorKey {
    /// Creates a fresh 256-bit credential. There is no deterministic fallback.
    ///
    /// # Errors
    /// OS randomness failures prevent daemon startup.
    pub fn generate() -> Result<Self, String> {
        use std::fmt::Write as _;
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes).map_err(|_| "OS randomness failed".to_owned())?;
        let mut key = String::with_capacity(64);
        for byte in bytes {
            write!(key, "{byte:02x}").map_err(|error| error.to_string())?;
        }
        Ok(Self(key))
    }

    /// Displays the credential only in the explicitly started operator terminal.
    #[must_use]
    pub fn for_operator_terminal(&self) -> &str {
        &self.0
    }

    fn accepts(&self, proof: Option<&str>) -> bool {
        proof.is_some_and(|proof| bool::from(self.0.as_bytes().ct_eq(proof.as_bytes())))
    }
}

/// A single daemon authority. The held file lock prevents duplicate writers.
pub struct Service {
    store: Store,
    state: PathBuf,
    operator: OperatorKey,
    keys: Option<shiin_audit::keys::Keys>,
    _lock: File,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resolve {
    proof: String,
    ticket: String,
    resolution: Resolution,
    lifetime: u64,
    scope: Option<GrantScope>,
}

impl Service {
    /// Opens protected daemon state. Existing corruption is never reset.
    ///
    /// # Errors
    /// Missing permissions, lock contention, malformed state, or failed entropy deny startup.
    pub fn open(directory: &Path, operator: OperatorKey) -> Result<Self, String> {
        let lock_path = directory.join("daemon.lock");
        let lock = private_file(&lock_path, false)?;
        lock.try_lock()
            .map_err(|_| "another daemon owns the state lock".to_owned())?;
        let state = directory.join("approvals.json");
        let store = match fs::read(&state) {
            Ok(bytes) if bytes.len() <= 32 * 1024 * 1024 => {
                serde_json::from_slice::<Store>(&bytes).map_err(|error| error.to_string())?
            }
            Ok(_) => return Err("approval store exceeds size limit".to_owned()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Store::default(),
            Err(error) => return Err(error.to_string()),
        };
        store.validate()?;
        let keys = if std::env::var("SHIIN_EVIDENCE").as_deref() == Ok("1") {
            Some(shiin_audit::keys::Keys::active()?)
        } else {
            None
        };
        Ok(Self {
            store,
            state,
            operator,
            keys,
            _lock: lock,
        })
    }

    /// Processes a JSON-RPC request after transport-level peer authentication.
    ///
    /// Operator methods always require the out-of-band credential; caller-supplied
    /// roles, asserted identities, and evaluation requests cannot resolve approvals.
    ///
    /// # Errors
    /// Every malformed input or failed transaction returns an error, never an allow.
    pub fn dispatch(&mut self, method: &str, params: Value) -> Result<Value, String> {
        match method {
            "shiin.evaluate" => self.evaluate(
                serde_json::from_value(params).map_err(|error| error.to_string())?,
                Mode::Attended,
            ),
            "shiin.evaluate.unattended" => self.evaluate(
                serde_json::from_value(params).map_err(|error| error.to_string())?,
                Mode::Unattended,
            ),
            "shiin.policy.validate" => {
                let text = params
                    .get("source")
                    .and_then(Value::as_str)
                    .ok_or("policy source missing")?;
                let root = params
                    .get("workspace")
                    .and_then(Value::as_str)
                    .ok_or("workspace missing")?;
                CompiledPolicy::compile(
                    &toml_front_end::parse(text).map_err(|error| error.to_string())?,
                    root,
                )
                .map_err(|error| error.to_string())?;
                Ok(json!({"valid":true}))
            }
            "shiin.pending" | "shiin.grants" | "shiin.approve.resolve" | "shiin.revoke" => {
                if !self
                    .operator
                    .accepts(params.get("proof").and_then(Value::as_str))
                {
                    return Err("operator authentication required".to_owned());
                }
                self.operator_method(method, params)
            }
            _ => Err("unknown method".to_owned()),
        }
    }

    fn operator_method(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let now = clock::now_unix_seconds();
        match method {
            "shiin.pending" => {
                let ticket = params.get("ticket").and_then(Value::as_str);
                let offset = params.get("offset").and_then(Value::as_u64).unwrap_or(0);
                let offset = usize::try_from(offset).map_err(|_| "invalid pending offset")?;
                Ok(json!(
                    self.store
                        .pending(now)
                        .into_iter()
                        .filter(|item| ticket.is_none_or(|ticket| item.approval.ticket == ticket))
                        .skip(offset)
                        .take(3)
                        .collect::<Vec<_>>()
                ))
            }
            "shiin.grants" => Ok(json!(self.store.grants())),
            "shiin.approve.resolve" | "shiin.revoke" => {
                let id = params
                    .get(if method == "shiin.revoke" {
                        "id"
                    } else {
                        "ticket"
                    })
                    .and_then(Value::as_str)
                    .ok_or("ticket or grant id missing")?
                    .to_owned();
                let workspace = self
                    .store
                    .workspace_for(&id)
                    .ok_or("unknown ticket or grant")?
                    .to_owned();
                let _workspace_lock = workspace_lock(Path::new(&workspace))?;
                let mut next = self.store.clone();
                let events = if method == "shiin.revoke" {
                    vec![next.revoke(&id)?]
                } else {
                    let resolution: Resolve =
                        serde_json::from_value(params).map_err(|error| error.to_string())?;
                    if !self.operator.accepts(Some(&resolution.proof)) {
                        return Err("operator authentication required".to_owned());
                    }
                    next.resolve(
                        &resolution.ticket,
                        resolution.resolution,
                        resolution.scope,
                        resolution.lifetime,
                        now,
                    )?
                };
                let log = AuditLog::new(Path::new(&workspace).join(".shiin/audit.jsonl"));
                for event in &events {
                    audit(&log, event.kind, &json!({"id":event.id}))?;
                }
                persist(
                    &self.state,
                    &serde_json::to_vec(&next).map_err(|error| error.to_string())?,
                )?;
                self.store = next;
                Ok(json!({"recorded":true}))
            }
            _ => Err("unknown operator method".to_owned()),
        }
    }

    fn evaluate(&mut self, mut request: ActionRequest, mode: Mode) -> Result<Value, String> {
        validate_evaluation_request(&request)?;
        let root = Path::new(&request.workspace.root);
        let _workspace_lock = workspace_lock(root)?;
        let policy_path = root.join(".shiin/policy.toml");
        let policy_bytes = fs::read(&policy_path).map_err(|error| error.to_string())?;
        if policy_bytes.len() > 1024 * 1024 {
            return Err("policy exceeds size limit".to_owned());
        }
        let source = toml_front_end::parse(
            std::str::from_utf8(&policy_bytes).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        let snapshot = source
            .layers
            .first()
            .ok_or("policy snapshot missing")?
            .digest
            .clone();
        if request.policy_snapshot != snapshot {
            return Err("policy snapshot changed; retry with current policy".to_owned());
        }
        let compiled = CompiledPolicy::compile(&source, &request.workspace.root)
            .map_err(|error| error.to_string())?;
        let task: Option<TaskDeclaration> = read_optional(&root.join(".shiin/tasks/active.json"))?;
        request.task = task.as_ref().map(|task| task.id.clone());
        let usage_path = root.join(".shiin/tasks/active.usage.json");
        let usage = if task.is_some() {
            let value: Value =
                read_optional(&usage_path)?.ok_or("active task counters are missing")?;
            for field in ["files_written", "files_deleted", "commands"] {
                if value.get(field).and_then(Value::as_u64).is_none() {
                    return Err("active task counters are damaged".to_owned());
                }
            }
            serde_json::from_value(value).map_err(|error| error.to_string())?
        } else {
            TaskUsage::default()
        };
        let now = clock::now_unix_seconds();
        let task_deadline = task
            .as_ref()
            .and_then(|task| task.expires_at.as_deref())
            .map(deadline)
            .transpose()?;
        let mut decision = decide(&Inputs {
            request: &request,
            policy: &compiled,
            source: &source,
            task: task.as_ref(),
            usage,
            now: &clock::rfc3339_from_unix_seconds(now),
            mode,
        });
        let mut next = self.store.clone();
        let transition = next.apply(&request, &mut decision, now, task_deadline)?;
        let log = AuditLog::new(root.join(".shiin/audit.jsonl"));
        audit(
            &log,
            "request.received",
            &serde_json::to_value(&request).map_err(|error| error.to_string())?,
        )?;
        if let Some(event) = transition {
            audit(
                &log,
                event.kind,
                &json!({"id":event.id, "request_id":request.id}),
            )?;
        }
        let mut body = serde_json::to_value(&decision).map_err(|error| error.to_string())?;
        body["policy_snapshot"] = json!(snapshot);
        audit(&log, "decision.made", &body)?;
        if let Some(keys) = &self.keys {
            self.record_evidence(
                keys,
                &request,
                &decision,
                &source,
                task.as_ref(),
                usage,
                now,
                mode,
                &log,
            )?;
        }
        if task.is_some() && decision.decision == shiin_schema::DecisionValue::Allow {
            persist(
                &usage_path,
                &serde_json::to_vec(&usage.plus(shiin_engine::usage_of(&request)))
                    .map_err(|error| error.to_string())?,
            )?;
        }
        persist(
            &self.state,
            &serde_json::to_vec(&next).map_err(|error| error.to_string())?,
        )?;
        self.store = next;
        serde_json::to_value(decision).map_err(|error| error.to_string())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the replay record captures each authoritative evaluation input explicitly"
    )]
    fn record_evidence(
        &self,
        keys: &shiin_audit::keys::Keys,
        request: &ActionRequest,
        decision: &shiin_schema::Decision,
        policy: &shiin_schema::Policy,
        task: Option<&TaskDeclaration>,
        usage: TaskUsage,
        now: u64,
        mode: Mode,
        log: &AuditLog,
    ) -> Result<(), String> {
        use sha2::{Digest, Sha256};
        use shiin_audit::receipts::{self, CHECKPOINT_TYPE, RECEIPT_TYPE, REPLAY_TYPE};
        let root = Path::new(&request.workspace.root).join(".shiin");
        let directory = root.join("evidence");
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let now_text = clock::rfc3339_from_unix_seconds(now);
        let request_digest = format!(
            "sha256:{:x}",
            Sha256::digest(
                shiin_audit::canonical_bytes(
                    &serde_json::to_value(request).map_err(|error| error.to_string())?
                )
                .map_err(|error| error.to_string())?
            )
        );
        let receipt = receipts::sign(
            &keys.signing,
            RECEIPT_TYPE,
            &json!({
                "schema":"shiin.receipt/v1-draft.3", "key_id":keys.id(), "request_id":request.id,
                "decision":decision.decision, "reason":decision.reason, "policy_snapshot":request.policy_snapshot,
                "request_digest":request_digest, "signed_at":now_text
            }),
        )?;
        let replay = json!({"schema":"shiin.replay-input/v1-draft.1", "request":request, "policy":policy, "task":task,
            "usage":usage, "now":now_text, "mode":mode, "decision":decision, "grants":self.store, "engine_version":shiin_engine::ENGINE_VERSION});
        let encrypted = keys.encrypt(
            &request.id,
            &shiin_audit::canonical_bytes(&replay).map_err(|error| error.to_string())?,
        )?;
        let replay_envelope = receipts::sign(
            &keys.signing,
            REPLAY_TYPE,
            &serde_json::to_value(encrypted).map_err(|error| error.to_string())?,
        )?;
        let events = log.read().map_err(|error| error.to_string())?;
        let last = events.last().ok_or("audit head missing")?;
        let checkpoint = receipts::sign(
            &keys.signing,
            CHECKPOINT_TYPE,
            &json!({
                "schema":"shiin.checkpoint/v1-draft.1", "key_id":keys.id(), "seq":last.seq, "head":last.hash, "signed_at":now_text
            }),
        )?;
        persist(
            &directory.join(format!("{}.receipt.json", request.id)),
            &serde_json::to_vec(&receipt).map_err(|error| error.to_string())?,
        )?;
        persist(
            &directory.join(format!("{}.replay.json", request.id)),
            &serde_json::to_vec(&replay_envelope).map_err(|error| error.to_string())?,
        )?;
        persist(
            &directory.join("checkpoint.json"),
            &serde_json::to_vec(&checkpoint).map_err(|error| error.to_string())?,
        )?;
        persist(
            &directory.join(format!(
                "{}.public.json",
                keys.id().trim_start_matches("sha256:")
            )),
            &serde_json::to_vec(&json!({"key_id":keys.id(), "public_key":base64_public(keys)}))
                .map_err(|error| error.to_string())?,
        )?;
        Ok(())
    }
}

fn validate_evaluation_request(request: &ActionRequest) -> Result<(), String> {
    request.validate().map_err(str::to_owned)?;
    if serde_json::to_vec(request)
        .map_err(|error| error.to_string())?
        .len()
        > shiin_ipc::MAX_FRAME / 4
    {
        return Err("normalized action exceeds service limit".to_owned());
    }
    if shiin_classify::canonical_root(&request.workspace.root) != request.workspace.root {
        return Err("workspace must be canonical".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_evaluation_request;
    use shiin_schema::{
        ActionRequest, Agent, Assurance, Confidence, Effect, Host, Workspace, versions,
    };

    fn request(workspace: String) -> ActionRequest {
        ActionRequest {
            schema: versions::ACTION_REQUEST.to_owned(),
            id: "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b".to_owned(),
            session: "EXAMPLE-session".to_owned(),
            agent: Agent {
                id: "EXAMPLE-agent".to_owned(),
                assurance: Assurance::Asserted,
            },
            action_type: "fs.write".to_owned(),
            effects: vec![Effect {
                kind: "path".to_owned(),
                value: "EXAMPLE.txt".to_owned(),
                confidence: Confidence::Exact,
                action_type: None,
            }],
            task: None,
            workspace: Workspace {
                root: workspace,
                vcs: None,
            },
            policy_snapshot: format!("sha256:{}", "0".repeat(64)),
            host: Host {
                name: "EXAMPLE-host".to_owned(),
                tool: None,
            },
        }
    }

    #[test]
    fn evaluation_rejects_noncanonical_workspace_roots() {
        let root = shiin_classify::canonical_root(
            std::env::temp_dir()
                .to_str()
                .expect("the temporary directory is UTF-8"),
        );
        let noncanonical = format!("{root}/..");
        let error = validate_evaluation_request(&request(noncanonical))
            .expect_err("a parent segment cannot reach policy evaluation");
        assert_eq!(error, "workspace must be canonical");
    }

    #[test]
    fn evaluation_rejects_oversized_normalized_actions() {
        let root = shiin_classify::canonical_root(
            std::env::temp_dir()
                .to_str()
                .expect("the temporary directory is UTF-8"),
        );
        let mut oversized = request(root);
        oversized.effects[0].value = "x".repeat(shiin_ipc::MAX_FRAME / 4);
        let error = validate_evaluation_request(&oversized)
            .expect_err("an oversized action cannot consume daemon resources");
        assert_eq!(error, "normalized action exceeds service limit");
    }
}

fn base64_public(keys: &shiin_audit::keys::Keys) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(keys.public())
}

fn audit(log: &AuditLog, kind: &str, body: &Value) -> Result<(), String> {
    log.append(kind, &clock::now_rfc3339(), body)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn workspace_lock(root: &Path) -> Result<File, String> {
    let file = private_file(&root.join(".shiin/state.lock"), false)?;
    file.try_lock()
        .map_err(|_| "workspace is busy".to_owned())?;
    Ok(file)
}

fn read_optional<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    match fs::read(path) {
        Ok(bytes) if bytes.len() <= 65_536 => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| error.to_string()),
        Ok(_) => Err("state file exceeds size limit".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn private_file(path: &Path, exclusive: bool) -> Result<File, String> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err("refusing symlink state file".to_owned());
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if exclusive {
        options.create_new(true);
    } else {
        options.create(true).truncate(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(|error| error.to_string())
}

/// Atomically replaces a state file and syncs its parent before success.
///
/// # Errors
/// Any write, rename, or durability failure prevents an allow response.
pub fn persist(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let id = clock::new_id();
    if id.is_empty() {
        return Err("OS randomness failed".to_owned());
    }
    let temporary = path.with_extension(format!("{id}.tmp"));
    let result = (|| {
        let mut file = private_file(&temporary, true)?;
        file.write_all(bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::rename(&temporary, path).map_err(|error| error.to_string())?;
        if let Some(parent) = path.parent() {
            File::open(parent)
                .and_then(|file| file.sync_all())
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
