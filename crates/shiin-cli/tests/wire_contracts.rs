//! Checks real serialized values against the published wire schemas.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use jsonschema::{Draft, Registry, Validator};
use serde_json::{Value, json};
use shiin_engine::{Inputs, decide};
use shiin_policy::{CompiledPolicy, toml_front_end};
use shiin_schema::{ActionRequest, Mode, Policy, TaskDeclaration, TaskUsage, clock};

fn validator(name: &str) -> Result<Validator, Box<dyn std::error::Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/schemas");
    let mut builder = Registry::new();
    let mut selected = Value::Null;
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let schema: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
        let id = schema["$id"].as_str().ok_or("schema has no id")?;
        builder = builder.add(id, schema.clone())?;
        if path.file_name().is_some_and(|file| file == name) {
            selected = schema;
        }
    }
    assert!(selected.is_object(), "the requested schema exists");
    Ok(jsonschema::options()
        .with_draft(Draft::Draft202012)
        .with_registry(&builder.prepare()?)
        .should_validate_formats(true)
        .build(&selected)?)
}

fn assert_valid(schema: &Validator, value: &Value) {
    let errors: Vec<String> = schema
        .iter_errors(value)
        .map(|error| error.to_string())
        .collect();
    assert!(errors.is_empty(), "wire contract violations: {errors:?}");
}

#[test]
fn corpus_requests_and_engine_decisions_match_their_wire_schemas()
-> Result<(), Box<dyn std::error::Error>> {
    let requests = validator("action-request.v1.schema.json")?;
    let decisions = validator("decision.v1.schema.json")?;
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/test-vectors");
    let mut count = 0;
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let case: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
        let request: ActionRequest = serde_json::from_value(case["action_request"].clone())?;
        request.validate()?;
        assert_valid(&requests, &serde_json::to_value(&request)?);
        let policy: Policy = serde_json::from_value(case["policy"].clone())?;
        let task: Option<TaskDeclaration> = case
            .get("task")
            .filter(|value| value.is_object())
            .cloned()
            .map(serde_json::from_value)
            .transpose()?;
        let compiled = CompiledPolicy::compile(&policy, &request.workspace.root)?;
        let decision = decide(&Inputs {
            request: &request,
            policy: &compiled,
            source: &policy,
            task: task.as_ref(),
            usage: TaskUsage::default(),
            now: "2026-10-15T00:00:00Z",
            mode: Mode::Attended,
        });
        assert_valid(&decisions, &serde_json::to_value(&decision)?);
        let trace = decision.trace.as_ref().ok_or("decision has no trace")?;
        assert_eq!(
            trace
                .iter()
                .map(|entry| entry.stage.as_str())
                .collect::<Vec<_>>(),
            ["identity", "capability", "policy", "intent", "risk"]
        );
        count += 1;
    }
    assert!(count >= 19, "all current corpus cases were exercised");
    Ok(())
}

#[test]
fn request_validation_rejects_unsupported_and_missing_wire_values()
-> Result<(), Box<dyn std::error::Error>> {
    let case: Value = serde_json::from_str(include_str!(
        "../../../docs/spec/test-vectors/tv-001-allowed-edit.v1.json"
    ))?;
    let original = &case["action_request"];
    for (field, value) in [
        ("schema", json!("shiin.action-request/v999")),
        ("id", json!("0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b")),
        ("effects", json!([])),
        ("session", json!("")),
        ("workspace", json!({"root": "relative"})),
        ("policy_snapshot", json!("sha256:EXAMPLE")),
    ] {
        let mut malformed = original.clone();
        malformed[field] = value;
        let request: ActionRequest = serde_json::from_value(malformed)?;
        assert!(request.validate().is_err(), "invalid {field} was accepted");
    }
    let mut unknown = original.clone();
    unknown["unexpected"] = json!(true);
    assert!(serde_json::from_value::<ActionRequest>(unknown).is_err());
    Ok(())
}

#[test]
fn risk_configuration_typos_fail_closed() {
    for suffix in [
        "[risk]\nopaque_commmand = 'pending'",
        "[risk]\nbulk_delete = { decision = 'pending', unknown_threshold = 20 }",
        "[risk]\nbulk_delete = { decision = 'pending', threshold = 0 }",
    ] {
        assert!(toml_front_end::parse(&format!("version = 1\n{suffix}\n")).is_err());
    }
}

#[test]
fn documented_full_policies_match_the_parser_and_schema() -> Result<(), Box<dyn std::error::Error>>
{
    let schema = validator("policy-syntax.v1.schema.json")?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let mut count = 0;
    for page in [
        "getting-started/first-policy.md",
        "guides/writing-policies.md",
        "concepts/how-shiin-works.md",
    ] {
        let text = fs::read_to_string(root.join(page))?.replace("\r\n", "\n");
        for block in text.split("```toml\n").skip(1) {
            let body = block
                .split("```")
                .next()
                .ok_or("unterminated TOML example")?;
            if !body
                .lines()
                .any(|line| line.trim_start().starts_with("version = 1"))
            {
                continue;
            }
            let value: Value = toml::from_str(body)?;
            assert_valid(&schema, &value);
            let policy = toml_front_end::parse(body)?;
            CompiledPolicy::compile(&policy, "/home/dev/project")?;
            count += 1;
        }
    }
    assert_eq!(count, 3, "each documented complete policy was checked");
    Ok(())
}

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Result<Self, std::io::Error> {
        let path = std::env::temp_dir().join(format!("shiin-contract-{}", clock::new_id()));
        fs::create_dir_all(path.join(".shiin"))?;
        fs::write(
            path.join(".shiin/policy.toml"),
            "version = 1\n[[rule]]\nid = 'commands'\neffect = 'allow'\naction_types = ['shell.exec']\n[risk]\nopaque_command = 'pending'\n",
        )?;
        Ok(Self(path))
    }

    fn hook(&self, host: &str, payload: &Value) -> Result<Value, Box<dyn std::error::Error>> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_shiin"))
            .args(["hook", host])
            .current_dir(&self.0)
            .env_remove("CI")
            .env_remove("SHIIN_UNATTENDED")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("hook stdin missing")?
            .write_all(payload.to_string().as_bytes())?;
        let output = child.wait_with_output()?;
        assert!(
            output.status.success(),
            "the host receives a protocol response"
        );
        Ok(serde_json::from_slice(&output.stdout)?)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn real_hook_events_conform_and_do_not_store_command_secrets()
-> Result<(), Box<dyn std::error::Error>> {
    let schema = validator("audit-events.v1.schema.json")?;
    let workspace = Workspace::new()?;
    let payload = json!({"session_id": "EXAMPLE-session", "tool_name": "Bash",
        "tool_input": {"command": "echo sk-EXAMPLE-DO-NOT-LOG"}});
    let response = workspace.hook("claude-code", &payload)?;
    assert_eq!(
        response["hookSpecificOutput"]["permissionDecision"],
        "allow"
    );
    let path = workspace.0.join(".shiin/audit.jsonl");
    let text = fs::read_to_string(&path)?;
    assert!(!text.contains("sk-EXAMPLE-DO-NOT-LOG"));
    let mut count = 0;
    for line in text.lines() {
        let event: Value = serde_json::from_str(line)?;
        assert_valid(&schema, &event);
        count += 1;
    }
    assert_eq!(count, 2);
    assert!(shiin_audit::AuditLog::new(&path).verify()?.is_empty());

    fs::write(&path, text.replacen("\"allow\"", "\"deny\"", 1))?;
    let denied = workspace.hook("claude-code", &payload)?;
    assert_eq!(denied["hookSpecificOutput"]["permissionDecision"], "deny");
    assert_eq!(
        fs::read_to_string(path)?.lines().count(),
        2,
        "a corrupted log is never extended"
    );
    Ok(())
}

#[test]
fn adapter_failure_responses_do_not_disclose_internal_details()
-> Result<(), Box<dyn std::error::Error>> {
    let detail = "/home/dev/project/.shiin/policy.toml SHIIN-POL-0001 sk-EXAMPLE-PRIVATE";
    let responses = [
        serde_json::to_string(&shiin_hooks::claude_code::HookResponse::fail_closed(detail))?,
        serde_json::to_string(&shiin_hooks::cursor::HookResponse::fail_closed(detail))?,
    ];
    for response in responses {
        assert!(response.contains("deny"));
        assert!(!response.contains("SHIIN-POL"));
        assert!(!response.contains("/home/dev"));
        assert!(!response.contains("EXAMPLE-PRIVATE"));
    }
    Ok(())
}

#[test]
fn pending_is_held_or_denied_by_each_adapter() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Workspace::new()?;
    for host in ["claude-code", "codex", "cursor"] {
        let payload = if host == "cursor" {
            json!({"conversation_id":"EXAMPLE-session", "hook_event_name":"beforeShellExecution",
                "command":"python EXAMPLE-script.py"})
        } else {
            json!({"session_id":"EXAMPLE-session", "tool_name":"Bash",
                "tool_input":{"command":"python EXAMPLE-script.py"}})
        };
        let response = workspace.hook(host, &payload)?;
        if host == "cursor" {
            assert_eq!(response["permission"], "deny");
        } else {
            let expected = if host == "claude-code" { "ask" } else { "deny" };
            assert_eq!(
                response["hookSpecificOutput"]["permissionDecision"],
                expected
            );
        }
        assert!(!response.to_string().contains("pending.opaque"));
    }
    Ok(())
}
