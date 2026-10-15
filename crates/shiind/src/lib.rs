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
        Ok(Self {
            store,
            state,
            operator,
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
        request.validate().map_err(str::to_owned)?;
        if serde_json::to_vec(&request)
            .map_err(|error| error.to_string())?
            .len()
            > shiin_ipc::MAX_FRAME / 4
        {
            return Err("normalized action exceeds service limit".to_owned());
        }
        let root = Path::new(&request.workspace.root);
        if shiin_classify::canonical_root(&request.workspace.root) != request.workspace.root {
            return Err("workspace must be canonical".to_owned());
        }
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
