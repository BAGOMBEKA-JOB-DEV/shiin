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

const POLICY: &str = "version = 1\n[capabilities.EXAMPLE]\nid = 'EXAMPLE'\nallow = true\naction_types = ['*']\n[defaults]\ndecision = 'pending'\n[[rule]]\nid = 'source'\neffect = 'allow'\naction_types = ['fs.write']\nresources = ['./src/**']\n";

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

fn serve_requests(
    mut service: Service,
    listener: &UnixListener,
    calls: usize,
) -> Result<(), String> {
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(15);
    for _ in 0..calls {
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
        stream.set_nonblocking(false).map_err(|e| e.to_string())?;
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
    let server = std::thread::spawn(move || serve_requests(service, &listener, 7));
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
        assert_eq!(denied.reason, "deny.policy");
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

#[test]
fn custom_agent_grants_require_operator_proof_and_preserve_deny()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = Workspace::new()?;
    let socket = workspace.0.join("s");
    let listener = UnixListener::bind(&socket)?;
    let operator = OperatorKey::generate()?;
    let proof = operator.for_operator_terminal().to_owned();
    let service = Service::open(&workspace.0.join("daemon"), operator)?;
    let server = std::thread::spawn(move || serve_requests(service, &listener, 14));
    let client = Client::at(&socket)?;
    let runtime = shiin_ipc::RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let mut action = workspace.action("migrations/EXAMPLE.sql")?;
        let pending = client.evaluate(&action).await?;
        assert_eq!(pending.decision, DecisionValue::Pending);
        let ticket = pending.approval.ok_or("approval ticket missing")?.ticket;
        approve_once(&socket, &proof, &ticket).await?;
        // A changed action cannot spend the exact grant.
        let changed = client
            .evaluate(&workspace.action("migrations/OTHER-EXAMPLE.sql")?)
            .await?;
        assert_eq!(changed.decision, DecisionValue::Pending);
        action.id = clock::new_id();
        assert_eq!(client.evaluate(&action).await?.reason, "allow.grant");
        action.id = clock::new_id();
        let spent = client.evaluate(&action).await?;
        assert_eq!(spent.decision, DecisionValue::Pending);
        let ticket = spent.approval.ok_or("retry ticket missing")?.ticket;
        shiin_ipc::call_at_async(
            &socket,
            "shiin.approve.resolve",
            json!({
                "proof":proof,"ticket":ticket,"resolution":"allow_always","lifetime":60,
                "scope":{"agent":action.agent.id,"workspace":workspace.0,
                    "action_types":["fs.write"],"resource":format!("{}/**",workspace.0.display())}
            }),
        )
        .await?;
        action.id = clock::new_id();
        assert_eq!(client.evaluate(&action).await?.reason, "allow.grant");
        let protected = client
            .evaluate(&workspace.action(".shiin/policy.toml")?)
            .await?;
        assert_eq!(protected.decision, DecisionValue::Deny);
        assert_eq!(protected.reason, "deny.policy");
        let other = workspace.action("migrations/OTHER-EXAMPLE.sql")?;
        assert_eq!(client.evaluate(&other).await?.reason, "allow.grant");
        let grants =
            shiin_ipc::call_at_async(&socket, "shiin.grants", json!({"proof":proof})).await?;
        let scoped = grants
            .as_array()
            .ok_or("grants must be an array")?
            .iter()
            .find(|grant| grant["scope"].is_object())
            .ok_or("scoped grant missing")?;
        shiin_ipc::call_at_async(
            &socket,
            "shiin.revoke",
            json!({"proof":proof,"id":scoped["id"]}),
        )
        .await?;
        assert_eq!(
            client.evaluate(&other).await?.decision,
            DecisionValue::Pending
        );
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    server.join().map_err(|_| "test service panicked")??;
    let audit = fs::read_to_string(workspace.0.join(".shiin/audit.jsonl"))?;
    assert!(
        !audit.contains(&proof),
        "operator proof entered the audit log"
    );
    for event in [
        "approval.resolved",
        "grant.created",
        "grant.used",
        "grant.revoked",
    ] {
        assert!(audit.contains(event), "{event} audit event missing");
    }
    Ok(())
}

async fn approve_once(
    socket: &std::path::Path,
    proof: &str,
    ticket: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resolve = json!({"ticket":ticket,"resolution":"allow_once","lifetime":60,"scope":null});
    assert!(
        shiin_ipc::call_at_async(socket, "shiin.approve.resolve", resolve.clone())
            .await
            .is_err()
    );
    let mut forged = resolve.clone();
    forged["proof"] = json!("EXAMPLE-invalid-operator-proof");
    forged["role"] = json!("operator");
    assert!(
        shiin_ipc::call_at_async(socket, "shiin.approve.resolve", forged)
            .await
            .is_err()
    );
    let mut authenticated = resolve;
    authenticated["proof"] = json!(proof);
    shiin_ipc::call_at_async(socket, "shiin.approve.resolve", authenticated).await?;
    Ok(())
}

#[test]
fn caller_task_claims_cannot_replace_the_daemons_active_task()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = Workspace::new()?;
    let policy = POLICY.replace("resources =", "tasks = ['EXAMPLE-task']\nresources =");
    fs::write(workspace.0.join(".shiin/policy.toml"), &policy)?;
    let socket = workspace.0.join("s");
    let listener = UnixListener::bind(&socket)?;
    let service = Service::open(&workspace.0.join("daemon"), OperatorKey::generate()?)?;
    let server = std::thread::spawn(move || serve_requests(service, &listener, 2));
    let client = Client::at(&socket)?;
    let mut action = workspace.action("src/EXAMPLE.rs")?;
    action.policy_snapshot = shiin_policy::toml_front_end::digest(&policy);
    action.task = Some("EXAMPLE-task".to_owned());
    let runtime = shiin_ipc::RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()?;
    let claimed = runtime.block_on(client.evaluate(&action))?;
    assert_eq!(claimed.decision, DecisionValue::Pending);
    fs::create_dir_all(workspace.0.join(".shiin/tasks"))?;
    fs::write(
        workspace.0.join(".shiin/tasks/active.json"),
        serde_json::to_vec(&json!({"id":"EXAMPLE-task"}))?,
    )?;
    fs::write(
        workspace.0.join(".shiin/tasks/active.usage.json"),
        serde_json::to_vec(&json!({"files_written":0,"files_deleted":0,"commands":0}))?,
    )?;
    action.id = clock::new_id();
    let active = runtime.block_on(client.evaluate(&action))?;
    assert_eq!(active.decision, DecisionValue::Allow);
    server.join().map_err(|_| "test service panicked")??;
    let usage: Value = serde_json::from_slice(&fs::read(
        workspace.0.join(".shiin/tasks/active.usage.json"),
    )?)?;
    assert_eq!(usage["files_written"], 1);
    Ok(())
}
