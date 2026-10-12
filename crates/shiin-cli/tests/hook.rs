//! End-to-end tests that run the real `shiin` binary as a host would.

#![expect(
    clippy::expect_used,
    reason = "a test may assume that its own fixtures are valid"
)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const POLICY: &str = r#"
version = 1

[defaults]
decision = "deny"

[[rule]]
id = "edit-source"
effect = "allow"
action_types = ["fs.read", "fs.write"]
resources = ["./src/**"]

[[rule]]
id = "no-secrets"
effect = "deny"
action_types = ["fs.read"]
resources = ["**/.env"]
reason = "Secret files are not readable by agents."

[risk]
opaque_command = "pending"
"#;

/// A workspace with a policy, unique to one test.
fn workspace(name: &str) -> PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!("shiin-cli-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".shiin")).expect("the workspace is created");
    fs::write(root.join(".shiin/policy.toml"), POLICY).expect("the policy is written");
    root
}

/// Runs the binary in `root`, feeding `payload` on standard input.
fn run_hook(root: &Path, payload: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_shiin"))
        .args(["hook", "claude-code"])
        .current_dir(root)
        .env_remove("CI")
        .env_remove("SHIIN_UNATTENDED")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the binary runs");
    child
        .stdin
        .as_mut()
        .expect("standard input is piped")
        .write_all(payload.as_bytes())
        .expect("the payload is written");
    let output = child.wait_with_output().expect("the binary finishes");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Runs a subcommand in `root` and returns its combined output.
fn run(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_shiin"))
        .args(arguments)
        .current_dir(root)
        .output()
        .expect("the binary runs");
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn payload(root: &Path, tool: &str, input: &str) -> String {
    format!(
        r#"{{"session_id":"sess-1","cwd":"{}","tool_name":"{tool}","tool_input":{input}}}"#,
        root.display()
    )
}

#[test]
fn a_policy_denial_reaches_the_host_as_a_denial() {
    let root = workspace("deny");
    let response = run_hook(&root, &payload(&root, "Read", r#"{"file_path":".env"}"#));
    assert!(
        response.contains("\"permissionDecision\":\"deny\""),
        "expected a denial, got: {response}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_allowed_edit_reaches_the_host_as_an_allow() {
    let root = workspace("allow");
    let response = run_hook(
        &root,
        &payload(&root, "Write", r#"{"file_path":"src/login.rs"}"#),
    );
    assert!(
        response.contains("\"permissionDecision\":\"allow\""),
        "expected an allow, got: {response}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_opaque_command_asks_a_human() {
    let root = workspace("ask");
    let response = run_hook(
        &root,
        &payload(&root, "Bash", r#"{"command":"python scripts/cleanup.py"}"#),
    );
    assert!(
        response.contains("\"permissionDecision\":\"ask\""),
        "expected an ask, got: {response}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_malformed_payload_denies_rather_than_allowing() {
    let root = workspace("malformed");
    let response = run_hook(&root, "this is not json");
    assert!(
        response.contains("\"permissionDecision\":\"deny\""),
        "a malformed payload must deny, got: {response}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_missing_policy_denies_rather_than_allowing() {
    let root = workspace("nopolicy");
    fs::remove_file(root.join(".shiin/policy.toml")).expect("the policy is removed");
    let response = run_hook(
        &root,
        &payload(&root, "Write", r#"{"file_path":"src/x.rs"}"#),
    );
    assert!(
        response.contains("\"permissionDecision\":\"deny\""),
        "a missing policy must deny, got: {response}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn decisions_are_recorded_and_the_chain_verifies() {
    let root = workspace("audit");
    run_hook(&root, &payload(&root, "Read", r#"{"file_path":".env"}"#));
    run_hook(
        &root,
        &payload(&root, "Write", r#"{"file_path":"src/login.rs"}"#),
    );

    let log = run(&root, &["log"]);
    assert!(log.contains("decision.made"), "log shows decisions: {log}");
    assert!(
        log.contains("deny (deny.policy)"),
        "log shows the denial: {log}"
    );

    let verified = run(&root, &["verify"]);
    assert!(
        verified.contains("intact"),
        "the chain verifies: {verified}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn validate_and_doctor_report_the_workspace() {
    let root = workspace("doctor");
    let validated = run(&root, &["policy", "validate"]);
    assert!(validated.contains("is valid"), "{validated}");

    let doctored = run(&root, &["doctor"]);
    assert!(doctored.contains("policy        ok"), "{doctored}");
    assert!(doctored.contains("L1 cooperative"), "{doctored}");
    let _ = fs::remove_dir_all(root);
}
