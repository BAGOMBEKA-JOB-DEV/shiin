//! Rust client for Shiin's local JSON-RPC API.
//!
//! Transport and daemon errors return [`ClientError`]. Callers must stop the
//! requested action on every error; this crate never converts a failed request
//! into `allow`.

use serde_json::{Value, json};
use shiin_schema::{ActionRequest, Decision};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors returned by the local client. None represent authorization to proceed.
#[derive(Debug, Error)]
pub enum ClientError {
    /// The daemon could not be reached or rejected the call.
    #[error("local authorization service failed: {0}")]
    Transport(String),
    /// A successful RPC returned a malformed result.
    #[error("local authorization service returned an invalid result")]
    InvalidResponse,
}

/// Asynchronous client bound to one absolute local daemon endpoint.
///
/// Create a client with [`Client::from_environment`] or [`Client::at`]. Calls
/// fail with an error when the daemon is unavailable, times out, or returns an
/// invalid response. The client does not expose operator-only approval methods.
pub struct Client {
    endpoint: PathBuf,
}

impl Client {
    /// Connects to the configured per-user daemon endpoint.
    ///
    /// # Errors
    /// Returns an error if the endpoint configuration is invalid.
    pub fn from_environment() -> Result<Self, ClientError> {
        let endpoint = shiin_ipc::endpoint().map_err(ClientError::Transport)?;
        Self::at(endpoint)
    }

    /// Creates a client for one absolute local daemon endpoint.
    ///
    /// # Errors
    /// Relative endpoints are rejected.
    pub fn at(endpoint: impl AsRef<Path>) -> Result<Self, ClientError> {
        let endpoint = endpoint.as_ref();
        if !endpoint.is_absolute() {
            return Err(ClientError::Transport(
                "local daemon endpoint must be absolute".to_owned(),
            ));
        }
        Ok(Self {
            endpoint: endpoint.to_path_buf(),
        })
    }

    /// Evaluates one normalized request through the daemon.
    ///
    /// # Errors
    /// Any transport failure or malformed decision returns an error. The caller
    /// must not execute the action unless the returned decision is `allow`.
    pub async fn evaluate(&self, request: &ActionRequest) -> Result<Decision, ClientError> {
        self.evaluate_method("shiin.evaluate", request).await
    }

    /// Evaluates one request in unattended mode, where `pending` becomes `deny`.
    ///
    /// # Errors
    /// Any transport failure or malformed decision returns an error.
    pub async fn evaluate_unattended(
        &self,
        request: &ActionRequest,
    ) -> Result<Decision, ClientError> {
        self.evaluate_method("shiin.evaluate.unattended", request)
            .await
    }

    /// Validates policy text against the daemon's policy compiler.
    ///
    /// # Errors
    /// Transport failures and invalid policy text return an error.
    pub async fn validate_policy(&self, policy: &str, workspace: &str) -> Result<(), ClientError> {
        let result = self
            .call(
                "shiin.policy.validate",
                json!({"policy":policy,"workspace":workspace}),
            )
            .await?;
        if result.get("valid") == Some(&Value::Bool(true)) {
            Ok(())
        } else {
            Err(ClientError::InvalidResponse)
        }
    }

    async fn evaluate_method(
        &self,
        method: &str,
        request: &ActionRequest,
    ) -> Result<Decision, ClientError> {
        request
            .validate()
            .map_err(|_| ClientError::InvalidResponse)?;
        let value = self
            .call(
                method,
                serde_json::to_value(request).map_err(|_| ClientError::InvalidResponse)?,
            )
            .await?;
        let decision: Decision =
            serde_json::from_value(value).map_err(|_| ClientError::InvalidResponse)?;
        if decision.request_id != request.id {
            return Err(ClientError::InvalidResponse);
        }
        Ok(decision)
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        shiin_ipc::call_at_async(&self.endpoint, method, params)
            .await
            .map_err(ClientError::Transport)
    }
}

#[cfg(test)]
mod tests {
    use super::{Client, ClientError};
    use shiin_schema::ActionRequest;

    #[test]
    fn rejects_relative_endpoint() {
        assert!(matches!(
            Client::at("relative.sock"),
            Err(ClientError::Transport(_))
        ));
    }

    #[test]
    fn unreachable_daemon_is_an_error() {
        let client = Client::at(test_socket_path("missing")).expect("absolute endpoint is valid");
        let request = request();
        let runtime = shiin_ipc::RuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .expect("create test runtime");
        assert!(matches!(
            runtime.block_on(client.evaluate(&request)),
            Err(ClientError::Transport(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn sends_request_and_returns_only_a_matching_decision() {
        use serde_json::{Value, json};
        use std::io::{BufRead as _, Write as _};
        use std::os::unix::net::UnixListener;
        use std::thread;

        let path = test_socket_path("roundtrip");
        let listener = UnixListener::bind(&path).expect("bind test endpoint");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept client");
            let mut reader = std::io::BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).expect("read request frame");
            let request: Value = serde_json::from_str(&line).expect("valid JSON-RPC request");
            let result = json!({
                "schema":"shiin.decision/v1-draft.1",
                "request_id":request["params"]["id"],
                "decision":"allow",
                "reason":"allow.policy",
                "message":"The action is allowed."
            });
            writeln!(
                reader.get_mut(),
                "{}",
                json!({"jsonrpc":"2.0","id":request["id"],"result":result})
            )
            .expect("write response frame");
        });

        let client = Client::at(&path).expect("construct client");
        let action = request();
        let runtime = shiin_ipc::RuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .expect("create test runtime");
        let decision = runtime
            .block_on(client.evaluate(&action))
            .expect("receive decision");
        assert_eq!(decision.request_id, action.id);
        assert_eq!(decision.decision, shiin_schema::DecisionValue::Allow);
        server.join().expect("server thread completes");
        std::fs::remove_file(path).expect("remove test socket");
    }

    #[cfg(unix)]
    #[test]
    fn preserves_deny_and_pending_and_rejects_mismatched_decision_ids() {
        use serde_json::{Value, json};
        use std::io::{BufRead as _, Write as _};
        use std::os::unix::net::UnixListener;
        use std::thread;

        for (index, (expected, expected_value)) in [
            ("deny", shiin_schema::DecisionValue::Deny),
            ("pending", shiin_schema::DecisionValue::Pending),
        ]
        .into_iter()
        .enumerate()
        {
            let path = test_socket_path(&format!("non-allow-{index}"));
            let listener = UnixListener::bind(&path).expect("bind test endpoint");
            let server = thread::spawn(move || {
                let (stream, _) = listener.accept().expect("accept client");
                let mut reader = std::io::BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).expect("read request frame");
                let request: Value = serde_json::from_str(&line).expect("valid JSON-RPC request");
                let decision = json!({
                    "schema":"shiin.decision/v1-draft.1",
                    "request_id":request["params"]["id"],
                    "decision":expected,
                    "reason":format!("{expected}.policy"),
                    "message":"The action must not proceed."
                });
                writeln!(
                    reader.get_mut(),
                    "{}",
                    json!({"jsonrpc":"2.0","id":request["id"],"result":decision})
                )
                .expect("write response frame");
            });
            let client = Client::at(&path).expect("construct client");
            let action = request();
            let runtime = shiin_ipc::RuntimeBuilder::new_current_thread()
                .enable_all()
                .build()
                .expect("create test runtime");
            let decision = runtime
                .block_on(client.evaluate(&action))
                .expect("receive decision");
            assert_eq!(decision.decision, expected_value);
            server.join().expect("server thread completes");
            std::fs::remove_file(path).expect("remove test socket");
        }

        let path = test_socket_path("mismatched-id");
        let listener = UnixListener::bind(&path).expect("bind test endpoint");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept client");
            let mut reader = std::io::BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).expect("read request frame");
            let request: Value = serde_json::from_str(&line).expect("valid JSON-RPC request");
            let result = json!({
                "schema":"shiin.decision/v1-draft.1",
                "request_id":"0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0c",
                "decision":"allow",
                "reason":"allow.policy",
                "message":"The action is allowed."
            });
            writeln!(
                reader.get_mut(),
                "{}",
                json!({"jsonrpc":"2.0","id":request["id"],"result":result})
            )
            .expect("write response frame");
        });
        let client = Client::at(&path).expect("construct client");
        let action = request();
        let runtime = shiin_ipc::RuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .expect("create test runtime");
        assert!(matches!(
            runtime.block_on(client.evaluate(&action)),
            Err(ClientError::InvalidResponse)
        ));
        server.join().expect("server thread completes");
        std::fs::remove_file(path).expect("remove test socket");
    }

    fn request() -> ActionRequest {
        serde_json::from_str(
            r#"{"schema":"shiin.action-request/v1-draft.1","id":"0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b","session":"session-EXAMPLE","agent":{"id":"agent-EXAMPLE","assurance":"asserted"},"action_type":"fs.read","effects":[{"kind":"path","value":"/tmp/EXAMPLE/file","confidence":"exact"}],"task":null,"workspace":{"root":"/tmp/EXAMPLE"},"policy_snapshot":"sha256:0000000000000000000000000000000000000000000000000000000000000000","host":{"name":"custom-agent"}}"#,
        )
        .expect("request is valid JSON")
    }

    fn test_socket_path(label: &str) -> std::path::PathBuf {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "shiin-client-{label}-{}-{}.sock",
            std::process::id(),
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ))
    }
}
