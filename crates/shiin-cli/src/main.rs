//! `shiin`, the command-line interface.
//!
//! The v0.1 commands cover the daemonless path: write a policy, validate it, decide
//! a request, serve a host hook, and read the audit log
//! ([ADR-0022](../../../docs/adr/0022-local-first-daemonless-deployment.md)).

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "shiin is a command-line tool that reports to the terminal"
)]

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use shiin_audit::AuditLog;
use shiin_engine::{Inputs, decide};
use shiin_hooks::{HookResponse, PreToolUse};
use shiin_policy::{CompiledPolicy, toml_front_end};
use shiin_schema::{ActionRequest, Decision, DecisionValue, Mode, Policy, clock};

/// Where a workspace keeps its policy.
const POLICY_PATH: &str = ".shiin/policy.toml";
/// Where a workspace keeps its audit log.
const AUDIT_PATH: &str = ".shiin/audit.jsonl";

/// The policy written by `shiin init`.
const STARTER_POLICY: &str = r#"# Shiin policy. See docs/guides/writing-policies.md
version = 1

# Anything no rule allows is denied.
[defaults]
decision = "deny"

[[rule]]
id = "read-and-edit-source"
effect = "allow"
action_types = ["fs.read", "fs.list", "fs.write", "fs.create"]
resources = ["./src/**", "./tests/**", "./docs/**"]

[[rule]]
id = "no-secrets"
effect = "deny"
action_types = ["fs.read"]
resources = ["**/.env", "**/.env.*", "~/.ssh/**", "**/credentials"]
reason = "Secret files are not readable by agents."

[[rule]]
id = "run-tests"
effect = "allow"
action_types = ["shell.exec"]
commands = ["cargo test*", "cargo nextest*", "git status", "git diff*"]

# Signals that raise a decision rather than deciding it.
[risk]
opaque_command = "pending"
secret_path = "deny"
bulk_delete = { decision = "pending", threshold = 20 }
destructive_vcs = "pending"
privilege_escalation = "pending"
"#;

fn main() -> ExitCode {
    // The host treats a crashed hook as an allow, so a panic must still deny.
    std::panic::set_hook(Box::new(|info| {
        let response = HookResponse::fail_closed("the hook stopped unexpectedly");
        if let Ok(text) = serde_json::to_string(&response) {
            println!("{text}");
        }
        eprintln!("shiin: internal error: {info}");
    }));

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["init"] => report(init()),
        ["policy", "validate"] => report(validate(Path::new(POLICY_PATH))),
        ["policy", "validate", path] => report(validate(Path::new(path))),
        ["check", path] => decide_file(path, false),
        ["explain", path] => decide_file(path, true),
        ["hook", "claude-code"] => hook_claude_code(),
        ["log"] => report(show_log(20)),
        ["log", count] => {
            let count = count.parse().unwrap_or(20);
            report(show_log(count))
        }
        ["verify"] => report(verify()),
        ["doctor"] => report(Ok(doctor())),
        ["version" | "--version" | "-V"] => report(Ok(version())),
        _ => {
            usage();
            ExitCode::FAILURE
        }
    }
}

/// What this build is, so a bug report can name it.
fn version() -> String {
    format!(
        "shiin {} (engine {}, pre-alpha: no releases yet)",
        env!("CARGO_PKG_VERSION"),
        shiin_engine::ENGINE_VERSION
    )
}

fn usage() {
    eprintln!("shiin: an authorization layer for AI agent actions (pre-alpha)");
    eprintln!();
    eprintln!("usage: shiin <command>");
    eprintln!();
    eprintln!("  init                      write a starter policy to {POLICY_PATH}");
    eprintln!("  policy validate [path]    parse and compile a policy");
    eprintln!("  check <request.json>      decide one action request");
    eprintln!("  explain <request.json>    decide, and show which stage decided");
    eprintln!("  hook claude-code          serve a Claude Code PreToolUse hook on stdin");
    eprintln!("  log [count]               show recent audit events");
    eprintln!("  verify                    check the audit log's hash chain");
    eprintln!("  doctor                    check this workspace's setup");
    eprintln!("  version                   print the version of this build");
}

/// Prints an outcome and turns it into an exit code.
fn report(outcome: Result<String, String>) -> ExitCode {
    match outcome {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("shiin: {message}");
            ExitCode::FAILURE
        }
    }
}

fn init() -> Result<String, String> {
    let path = Path::new(POLICY_PATH);
    if path.exists() {
        return Err(format!(
            "{POLICY_PATH} already exists, so nothing was written"
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    fs::write(path, STARTER_POLICY).map_err(|error| format!("{POLICY_PATH}: {error}"))?;
    Ok(format!(
        "wrote {POLICY_PATH}\nNothing enforces it yet: configure your host's hook to run `shiin hook claude-code`."
    ))
}

/// Reads and compiles the policy for this workspace.
fn load_policy(path: &Path, workspace_root: &str) -> Result<(Policy, CompiledPolicy), String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("SHIIN-POL-0001: {}: {error}", path.display()))?;
    let policy = toml_front_end::parse(&text).map_err(|error| described(&error))?;
    let compiled =
        CompiledPolicy::compile(&policy, workspace_root).map_err(|error| described(&error))?;
    Ok((policy, compiled))
}

/// Renders a policy error with its stable code, so a script can match on it.
fn described(error: &shiin_policy::PolicyError) -> String {
    format!("{}: {error}", error.code())
}

fn validate(path: &Path) -> Result<String, String> {
    let root = workspace_root();
    let (policy, _) = load_policy(path, &root)?;
    let rules = policy.rules().count();
    let signals = policy.risk.len();
    Ok(format!(
        "{} is valid: {rules} rule(s), {signals} risk signal(s), default {:?}",
        path.display(),
        policy.default
    ))
}

/// The workspace this command applies to.
fn workspace_root() -> String {
    std::env::var("SHIIN_WORKSPACE").unwrap_or_else(|_| {
        std::env::current_dir().map_or_else(
            |_| ".".to_owned(),
            |path| path.to_string_lossy().into_owned(),
        )
    })
}

/// Decides a request read from a file, optionally explaining the outcome.
fn decide_file(path: &str, explain: bool) -> ExitCode {
    let outcome = (|| -> Result<Decision, String> {
        let text = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
        let request: ActionRequest =
            serde_json::from_str(&text).map_err(|error| format!("{path}: {error}"))?;
        let (source, compiled) = load_policy(Path::new(POLICY_PATH), &request.workspace.root)?;
        Ok(decide(&Inputs {
            request: &request,
            policy: &compiled,
            source: &source,
            task: None,
            mode: mode(),
        }))
    })();

    match outcome {
        Ok(decision) => {
            if explain {
                println!("decision      {:?}", decision.decision);
                println!("reason        {}", decision.reason);
                println!("stage         {}", decision.stage);
                println!(
                    "rules         {}",
                    if decision.matched_rules.is_empty() {
                        "none".to_owned()
                    } else {
                        decision.matched_rules.join(", ")
                    }
                );
                println!("engine        {}", decision.engine_version);
            } else {
                println!("{:?} ({})", decision.decision, decision.reason);
            }
            match decision.decision {
                DecisionValue::Allow => ExitCode::SUCCESS,
                DecisionValue::Deny => ExitCode::FAILURE,
                DecisionValue::Pending => ExitCode::from(2),
            }
        }
        Err(message) => {
            eprintln!("shiin: {message}");
            // Failing to decide is a denial, never an allow.
            ExitCode::FAILURE
        }
    }
}

/// Whether an approver can be reached, taken from the environment.
fn mode() -> Mode {
    if std::env::var("SHIIN_UNATTENDED").is_ok() || std::env::var("CI").is_ok() {
        Mode::Unattended
    } else {
        Mode::Attended
    }
}

/// Serves one Claude Code `PreToolUse` hook call.
///
/// Every failure prints a deny response, because the host would otherwise allow the
/// action. The exit code stays zero so the host reads the response as a decision
/// rather than as a hook error.
fn hook_claude_code() -> ExitCode {
    let response = match serve_hook() {
        Ok(response) => response,
        Err(detail) => {
            eprintln!("shiin: {detail}");
            HookResponse::fail_closed(&detail)
        }
    };
    match serde_json::to_string(&response) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        // Without a response the host would allow the action, so block it outright.
        Err(_) => ExitCode::from(2),
    }
}

fn serve_hook() -> Result<HookResponse, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("cannot read the hook payload: {error}"))?;
    let payload: PreToolUse = serde_json::from_str(&input)
        .map_err(|error| format!("the hook payload is not valid: {error}"))?;

    let root = if payload.cwd.is_empty() {
        workspace_root()
    } else {
        payload.cwd.clone()
    };
    let mut request = shiin_hooks::to_action_request(&payload, &root);
    let (source, compiled) = load_policy(Path::new(POLICY_PATH), &root)?;
    request.policy_snapshot = source
        .layers
        .first()
        .map(|layer| layer.digest.clone())
        .unwrap_or_default();

    let decision = decide(&Inputs {
        request: &request,
        policy: &compiled,
        source: &source,
        task: None,
        mode: mode(),
    });

    // The decision is recorded before the action is allowed to proceed.
    record(&request, &decision)?;
    Ok(HookResponse::from_decision(&decision))
}

/// Appends the request and its decision to the audit log.
fn record(request: &ActionRequest, decision: &Decision) -> Result<(), String> {
    let log = AuditLog::new(AUDIT_PATH);
    if let Some(parent) = log.path().parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("SHIIN-AUD-0001: {}: {error}", parent.display()))?;
    }
    let now = clock::now_rfc3339();
    for (event_type, body) in [
        ("request.received", serde_json::to_value(request)),
        ("decision.made", serde_json::to_value(decision)),
    ] {
        let body = body.map_err(|error| format!("SHIIN-GEN-0001: {error}"))?;
        log.append(event_type, &now, body)
            .map_err(|error| format!("{}: {error}", error.code()))?;
    }
    Ok(())
}

fn show_log(count: usize) -> Result<String, String> {
    let events = AuditLog::new(AUDIT_PATH)
        .read()
        .map_err(|error| format!("{}: {error}", error.code()))?;
    if events.is_empty() {
        return Ok("the audit log is empty".to_owned());
    }
    let start = events.len().saturating_sub(count);
    let lines: Vec<String> = events[start..]
        .iter()
        .map(|event| {
            let detail = event.payload["decision"].as_str().map_or_else(
                || {
                    event.payload["action_type"]
                        .as_str()
                        .unwrap_or("")
                        .to_owned()
                },
                |decision| {
                    format!(
                        "{decision} ({})",
                        event.payload["reason"].as_str().unwrap_or("")
                    )
                },
            );
            format!(
                "{:>4}  {}  {:<18} {detail}",
                event.seq, event.time, event.event_type
            )
        })
        .collect();
    Ok(lines.join("\n"))
}

fn verify() -> Result<String, String> {
    let log = AuditLog::new(AUDIT_PATH);
    let events = log
        .read()
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let breaks = log
        .verify()
        .map_err(|error| format!("{}: {error}", error.code()))?;
    if breaks.is_empty() {
        Ok(format!(
            "the audit log is intact: {} event(s), chain verified",
            events.len()
        ))
    } else {
        Err(format!(
            "SHIIN-AUD-0002: the audit log does not verify\n{}",
            breaks
                .iter()
                .map(|chain_break| format!("  event {}: {}", chain_break.seq, chain_break.detail))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

fn doctor() -> String {
    let mut lines = Vec::new();
    let root = workspace_root();
    lines.push(format!("workspace     {root}"));

    let policy_path = PathBuf::from(POLICY_PATH);
    match load_policy(&policy_path, &root) {
        Ok((policy, _)) => lines.push(format!(
            "policy        ok, {} rule(s) in {POLICY_PATH}",
            policy.rules().count()
        )),
        Err(error) => lines.push(format!("policy        PROBLEM: {error}")),
    }

    let log = AuditLog::new(AUDIT_PATH);
    match log.verify() {
        Ok(breaks) if breaks.is_empty() => {
            lines.push("audit log     ok, chain verified".to_owned());
        }
        Ok(breaks) => lines.push(format!("audit log     PROBLEM: {} break(s)", breaks.len())),
        Err(error) => lines.push(format!("audit log     PROBLEM: {error}")),
    }

    lines.push(format!("mode          {:?}", mode()));
    lines.push(
        "enforcement   L1 cooperative: the host honors decisions, and its hooks can be turned off"
            .to_owned(),
    );
    lines.push(
        "reminder      Shiin only sees actions a host sends it. Pair it with a sandbox.".to_owned(),
    );
    lines.join("\n")
}
