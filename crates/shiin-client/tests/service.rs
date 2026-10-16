//! Exercises the client against the real service rather than fabricated decisions.

#![cfg(unix)]

use serde_json::{Value, json};
use shiin_client::Client;
use shiin_schema::{ActionRequest, DecisionValue, clock};
use shiind::{OperatorKey, Service};
use std::fs;
use std::io::{BufRead as _, Write as _};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const POLICY: &str = "version = 1\n[defaults]\ndecision = 'pending'\n[[rule]]\nid = 'source'\neffect = 'allow'\naction_types = ['fs.write']\nresources = ['./src/**']\n";

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("sc-{}", clock::new_id()));
        fs::create_dir_all(root.join(".shiin"))?;
        fs::create_dir_all(root.join("daemon"))?;
        fs::write(root.join(".shiin/policy.toml"), POLICY)?;
        Ok(Self(fs::canonicalize(root)?))
    }

    fn action(&self, path: &str) -> Result<ActionRequest, Box<dyn std::error::Error>> {
        let source = shiin_policy::toml_front_end::parse(POLICY)?;
        Ok(serde_json::from_value(json!({
            "schema":"shiin.action-request/v1-draft.1",
            "id":clock::new_id(), "session":"EXAMPLE-session",
            "agent":{"id":"EXAMPLE-agent","assurance":"asserted"},
            "action_type":"fs.write",
            "effects":[{"kind":"path","value":self.0.join(path),"confidence":"exact"}],
            "workspace":{"root":self.0}, "policy_snapshot":source.layers[0].digest,
            "host":{"name":"EXAMPLE-custom-agent"}
        }))?)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn serve_requests(mut service: Service, listener: &UnixListener) -> Result<(), String> {
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(15);
    for _ in 0..7 {
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err("test client did not complete its calls".to_owned());
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(error) => return Err(error.to_string()),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        let mut reader = std::io::BufReader::new(stream);
        let mut frame = String::new();
        reader.read_line(&mut frame).map_err(|e| e.to_string())?;
        let request: shiin_ipc::Request =
            serde_json::from_str(&frame).map_err(|e| e.to_string())?;
        let response = match service.dispatch(&request.method, request.params) {
            Ok(result) => json!({"jsonrpc":"2.0", "id":request.id, "result":result}),
            Err(_) => shiin_ipc::failure(&request.id, -32000, "Request refused"),
        };
        writeln!(reader.get_mut(), "{response}").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[test]
fn custom_agent_and_service_obey_the_local_contract() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Workspace::new()?;
    let socket = workspace.0.join("s");
    let listener = UnixListener::bind(&socket)?;
    let service = Service::open(&workspace.0.join("daemon"), OperatorKey::generate()?)?;
    let server = std::thread::spawn(move || serve_requests(service, &listener));
    let client = Client::at(&socket)?;
    let runtime = shiin_ipc::RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        client
            .validate_policy(
                POLICY,
                workspace.0.to_str().ok_or("workspace is not UTF-8")?,
            )
            .await?;
        assert!(
            client
                .validate_policy("invalid policy", "/EXAMPLE")
                .await
                .is_err()
        );
        let allowed = client
            .evaluate(&workspace.action("src/EXAMPLE.rs")?)
            .await?;
        assert_eq!(allowed.decision, DecisionValue::Allow);
        let denied = client
            .evaluate(&workspace.action(".shiin/policy.toml")?)
            .await?;
        assert_eq!(denied.decision, DecisionValue::Deny);
        assert_eq!(denied.reason, "deny.builtin");
        let action = workspace.action("migrations/EXAMPLE.sql")?;
        let pending = client.evaluate(&action).await?;
        assert_eq!(pending.decision, DecisionValue::Pending);
        assert!(pending.approval.is_some());
        let unattended = client.evaluate_unattended(&action).await?;
        assert_eq!(unattended.decision, DecisionValue::Deny);
        assert_eq!(unattended.reason, "deny.unattended");
        let mut stale = workspace.action("src/EXAMPLE.rs")?;
        stale.policy_snapshot = format!("sha256:{}", "0".repeat(64));
        assert!(client.evaluate(&stale).await.is_err());
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    server.join().map_err(|_| "test service panicked")??;
    let events: Vec<Value> = fs::read_to_string(workspace.0.join(".shiin/audit.jsonl"))?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    assert_eq!(
        events
            .iter()
            .filter(|e| e["kind"] == "decision.made")
            .count(),
        4
    );
    Ok(())
}
