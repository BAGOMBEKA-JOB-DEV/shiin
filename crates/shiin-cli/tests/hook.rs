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
    run_hook_full(root, payload).0
}

/// Runs the hook and returns its standard output and standard error.
fn run_hook_full(root: &Path, payload: &str) -> (String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_shiin"))
        .args(["hook", "claude-code"])
        .current_dir(root)
        .env_remove("CI")
        .env_remove("SHIIN_UNATTENDED")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");
    child
        .stdin
        .as_mut()
        .expect("standard input is piped")
        .write_all(payload.as_bytes())
        .expect("the payload is written");
    let output = child.wait_with_output().expect("the binary finishes");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
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
    let arguments: serde_json::Value =
        serde_json::from_str(input).expect("the tool arguments are valid JSON");
    serde_json::json!({
        "session_id": "sess-1",
        "cwd": root.to_string_lossy(),
        "tool_name": tool,
        "tool_input": arguments,
    })
    .to_string()
}

#[test]
// Path matching needs canonicalization, which Windows does not have yet:
// see ADR-0019 and docs/reference/platform-support.md.
#[cfg(unix)]
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
// Path matching needs canonicalization, which Windows does not have yet:
// see ADR-0019 and docs/reference/platform-support.md.
#[cfg(unix)]
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
// Path matching needs canonicalization, which Windows does not have yet:
// see ADR-0019 and docs/reference/platform-support.md.
#[cfg(unix)]
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

// Fault injection: every one of these would be an allow if Shiin failed open,
// because the host treats a hook that does not answer as permission to proceed.

#[test]
fn an_unparsable_policy_denies_and_names_its_code() {
    let root = workspace("badtoml");
    fs::write(root.join(".shiin/policy.toml"), "version = 1\n[[rule\n").expect("written");
    let (response, errors) = run_hook_full(
        &root,
        &payload(&root, "Write", r#"{"file_path":"src/x.rs"}"#),
    );
    assert!(
        response.contains("\"permissionDecision\":\"deny\""),
        "an unparsable policy must deny, got: {response}"
    );
    assert!(
        errors.contains("SHIIN-POL-0002"),
        "expected a syntax code: {errors}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_unknown_policy_key_denies_rather_than_being_ignored() {
    let root = workspace("unknownkey");
    fs::write(
        root.join(".shiin/policy.toml"),
        "version = 1\n[[rule]]\nid = \"r\"\neffect = \"allow\"\nsuprise = true\n",
    )
    .expect("written");
    let (response, errors) = run_hook_full(
        &root,
        &payload(&root, "Write", r#"{"file_path":"src/x.rs"}"#),
    );
    assert!(
        response.contains("\"permissionDecision\":\"deny\""),
        "an unknown key must deny, got: {response}"
    );
    assert!(
        errors.contains("SHIIN-POL-0003"),
        "expected an unknown-key code: {errors}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_unwritable_audit_log_denies_the_action() {
    let root = workspace("noaudit");
    // A directory where the log file belongs makes every append fail.
    fs::create_dir_all(root.join(".shiin/audit.jsonl")).expect("the obstacle is created");
    let (response, errors) = run_hook_full(
        &root,
        &payload(&root, "Write", r#"{"file_path":"src/x.rs"}"#),
    );
    assert!(
        response.contains("\"permissionDecision\":\"deny\""),
        "an action must not be allowed if it cannot be recorded, got: {response}"
    );
    assert!(
        errors.contains("SHIIN-AUD-0001"),
        "expected an audit code: {errors}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_empty_payload_denies() {
    let root = workspace("emptystdin");
    let response = run_hook(&root, "");
    assert!(
        response.contains("\"permissionDecision\":\"deny\""),
        "an empty payload must deny, got: {response}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_tampered_audit_log_fails_verification() {
    let root = workspace("tampered");
    run_hook(&root, &payload(&root, "Read", r#"{"file_path":".env"}"#));
    let log = root.join(".shiin/audit.jsonl");
    let text = fs::read_to_string(&log).expect("the log is readable");
    fs::write(&log, text.replacen("\"deny\"", "\"allow\"", 1)).expect("the log is rewritten");

    let verified = run(&root, &["verify"]);
    assert!(
        verified.contains("SHIIN-AUD-0002"),
        "tampering must be reported: {verified}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_decision_arrives_well_inside_a_host_timeout() {
    let root = workspace("latency");
    // Hosts allow a hook that does not answer, so answering late is a failure mode.
    // This is a smoke guard, not a benchmark: the budget lives in the engineering docs.
    for _ in 0..5_u8 {
        let started = std::time::Instant::now();
        let response = run_hook(
            &root,
            &payload(&root, "Write", r#"{"file_path":"src/x.rs"}"#),
        );
        let elapsed = started.elapsed();
        assert!(
            response.contains("permissionDecision"),
            "every call answers: {response}"
        );
        assert!(
            elapsed < std::time::Duration::from_secs(2),
            "a decision took {elapsed:?}, which is far too close to a host timeout"
        );
    }
    let _ = fs::remove_dir_all(root);
}

// Task declarations: the constraint that applies only to the work the person asked for.

#[test]
// Path matching needs canonicalization, which Windows does not have yet:
// see ADR-0019 and docs/reference/platform-support.md.
#[cfg(unix)]
fn a_task_scopes_and_budgets_an_agent_then_releases_it() {
    let root = workspace("task");
    // The policy allows writes anywhere in the workspace, so only the task narrows it.
    fs::write(
        root.join(".shiin/policy.toml"),
        "version = 1\n[defaults]\ndecision = \"deny\"\n\n[[rule]]\nid = \"writes\"\neffect = \"allow\"\naction_types = [\"fs.write\"]\nresources = [\"./**\"]\n",
    )
    .expect("the policy is written");

    let started = run(
        &root,
        &[
            "task",
            "start",
            "Fix the login bug",
            "--scope",
            "./src/auth/**",
            "--max-writes",
            "1",
        ],
    );
    assert!(started.contains("started"), "{started}");

    // Inside the scope and within budget.
    let first = run_hook(
        &root,
        &payload(&root, "Write", r#"{"file_path":"src/auth/login.rs"}"#),
    );
    assert!(
        first.contains("\"permissionDecision\":\"allow\""),
        "the first write is inside the task: {first}"
    );

    // The budget is now spent, even though policy still allows the write.
    let second = run_hook(
        &root,
        &payload(&root, "Write", r#"{"file_path":"src/auth/session.rs"}"#),
    );
    assert!(
        second.contains("\"permissionDecision\":\"deny\""),
        "the write limit is reached: {second}"
    );

    let shown = run(&root, &["task", "show"]);
    assert!(shown.contains("writes        1 of 1"), "{shown}");

    // Ending the task releases the constraint, and policy alone decides again.
    let ended = run(&root, &["task", "end"]);
    assert!(ended.contains("task ended"), "{ended}");
    let after = run_hook(
        &root,
        &payload(&root, "Write", r#"{"file_path":"docs/notes.md"}"#),
    );
    assert!(
        after.contains("\"permissionDecision\":\"allow\""),
        "with no task, policy decides: {after}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
// Path matching needs canonicalization, which Windows does not have yet:
// see ADR-0019 and docs/reference/platform-support.md.
#[cfg(unix)]
fn a_task_refuses_work_outside_its_scope() {
    let root = workspace("taskscope");
    fs::write(
        root.join(".shiin/policy.toml"),
        "version = 1\n[defaults]\ndecision = \"deny\"\n\n[[rule]]\nid = \"writes\"\neffect = \"allow\"\naction_types = [\"fs.write\"]\nresources = [\"./**\"]\n",
    )
    .expect("the policy is written");
    run(
        &root,
        &[
            "task",
            "start",
            "Fix the login bug",
            "--scope",
            "./src/auth/**",
        ],
    );

    let response = run_hook(
        &root,
        &payload(&root, "Write", r#"{"file_path":"migrations/0042.sql"}"#),
    );
    assert!(
        response.contains("\"permissionDecision\":\"deny\""),
        "a migration is not part of fixing the login bug: {response}"
    );
    // A refused action must not consume the task's budget.
    let shown = run(&root, &["task", "show"]);
    assert!(shown.contains("writes        0 of none"), "{shown}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_unknown_task_option_is_refused_rather_than_ignored() {
    let root = workspace("taskopt");
    let output = run(
        &root,
        &["task", "start", "Something", "--scpoe", "./src/**"],
    );
    assert!(output.contains("unknown option"), "{output}");
    let _ = fs::remove_dir_all(root);
}

// Other hosts speak their own protocols, and a decision has to arrive in each one.

#[test]
// Path matching needs canonicalization, which Windows does not have yet:
// see ADR-0019 and docs/reference/platform-support.md.
#[cfg(unix)]
fn codex_hears_a_decision_in_its_own_protocol() {
    let root = workspace("codex");
    let payload = serde_json::json!({
        "session_id": "s",
        "cwd": root.to_string_lossy(),
        "tool_name": "apply_patch",
        "tool_input": { "path": "src/login.rs" },
    })
    .to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_shiin"))
        .args(["hook", "codex"])
        .current_dir(&root)
        .env_remove("CI")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .as_mut()
                .expect("standard input is piped")
                .write_all(payload.as_bytes())?;
            child.wait_with_output()
        })
        .expect("the binary runs");
    let response = String::from_utf8_lossy(&output.stdout);
    assert!(
        response.contains("\"permissionDecision\":\"allow\""),
        "a patch inside src is allowed: {response}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
// Path matching needs canonicalization, which Windows does not have yet:
// see ADR-0019 and docs/reference/platform-support.md.
#[cfg(unix)]
fn cursor_hears_a_denial_for_a_secret_read_through_a_command() {
    let root = workspace("cursor");
    let payload = serde_json::json!({
        "hook_event_name": "beforeShellExecution",
        "conversation_id": "c",
        "workspace_roots": [root.to_string_lossy()],
        "command": "cat .env",
    })
    .to_string();
    let mut child = Command::new(env!("CARGO_BIN_EXE_shiin"))
        .args(["hook", "cursor"])
        .current_dir(&root)
        .env_remove("CI")
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
    let response = String::from_utf8_lossy(&output.stdout);
    assert!(
        response.contains("\"permission\":\"deny\""),
        "Cursor hears a denial in its own shape: {response}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
// Path matching needs canonicalization, which Windows does not have yet:
// see ADR-0019 and docs/reference/platform-support.md.
#[cfg(unix)]
fn policy_cases_pass_and_a_wrong_expectation_fails() {
    let root = workspace("policytest");
    fs::create_dir_all(root.join(".shiin/tests")).expect("the case directory is created");
    let case = |id: &str, decision: &str, reason: &str| {
        let workspace = root.to_string_lossy().into_owned();
        serde_json::json!({
            "schema": "shiin.test-vector/v1-draft.1",
            "id": id,
            "name": id,
            "action_request": {
                "schema": "shiin.action-request/v1-draft.1",
                "id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9aa3",
                "session": "m",
                "agent": { "id": "claude-code", "assurance": "asserted" },
                "action_type": "fs.read",
                "effects": [{
                    "kind": "path",
                    "value": format!("{workspace}/.env"),
                    "confidence": "exact",
                }],
                "workspace": { "root": workspace },
                "policy_snapshot": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                "host": { "name": "claude-code", "tool": "Read" },
            },
            "expected": { "decision": decision, "reason": reason },
        })
        .to_string()
    };
    fs::write(
        root.join(".shiin/tests/right.json"),
        case("right", "deny", "deny.policy"),
    )
    .expect("the passing case is written");
    let passing = run(&root, &["policy", "test"]);
    assert!(passing.contains("ok      right"), "{passing}");
    assert!(passing.contains("0 failed"), "{passing}");

    fs::write(
        root.join(".shiin/tests/wrong.json"),
        case("wrong", "allow", "allow.policy"),
    )
    .expect("the failing case is written");
    let failing = run(&root, &["policy", "test"]);
    assert!(failing.contains("FAILED  wrong"), "{failing}");
    let _ = fs::remove_dir_all(root);
}

#[test]
// Reproduces on any Unix what macOS does by default: its temporary directories live
// under /var, which is a symlink to /private/var.
#[cfg(unix)]
fn a_workspace_reached_through_a_symlink_still_matches_policy() {
    let real = workspace("symlink-real");
    let mut link = std::env::temp_dir();
    link.push(format!("shiin-cli-{}-symlink-view", std::process::id()));
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(&real, &link).expect("the symlink is created");

    // The host reports the path it was started with, which here runs through the link.
    let response = run_hook(
        &link,
        &payload(&link, "Write", r#"{"file_path":"src/login.rs"}"#),
    );
    assert!(
        response.contains("\"permissionDecision\":\"allow\""),
        "a root reached through a symlink must resolve to the same place as its effects: {response}"
    );
    let _ = fs::remove_file(link);
    let _ = fs::remove_dir_all(real);
}
