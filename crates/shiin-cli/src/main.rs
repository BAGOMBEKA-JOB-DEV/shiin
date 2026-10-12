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

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use shiin_audit::AuditLog;
use shiin_engine::{Inputs, decide, vectors};
use shiin_hooks::claude_code::{self, HookResponse, PreToolUse};
use shiin_hooks::{codex, cursor};
use shiin_policy::{CompiledPolicy, toml_front_end};
use shiin_schema::{
    ActionRequest, Decision, DecisionValue, Limits, Mode, Policy, Scope, TaskDeclaration,
    TaskUsage, clock,
};

/// Where a workspace keeps its policy.
const POLICY_PATH: &str = ".shiin/policy.toml";
/// Where a workspace keeps its audit log.
const AUDIT_PATH: &str = ".shiin/audit.jsonl";
/// Where a workspace keeps its policy cases.
const TEST_PATH: &str = ".shiin/tests";
/// Where a workspace keeps the active task declaration.
const TASK_PATH: &str = ".shiin/tasks/active.json";
/// Where a workspace keeps the active task's counters.
const USAGE_PATH: &str = ".shiin/tasks/active.usage.json";
/// Serializes task and hook state changes across processes.
const LOCK_PATH: &str = ".shiin/state.lock";
/// Internal deadline, including reading the host's input.
const HOOK_TIMEOUT: Duration = Duration::from_secs(2);

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
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let panic_host = if words.as_slice() == ["hook", "cursor"] {
        HostKind::Cursor
    } else if words.as_slice() == ["hook", "codex"] {
        HostKind::Codex
    } else {
        HostKind::ClaudeCode
    };
    install_panic_hook(panic_host);
    match words.as_slice() {
        ["init"] => report(init()),
        ["policy", "validate"] => report(validate(Path::new(POLICY_PATH))),
        ["policy", "validate", path] => report(validate(Path::new(path))),
        ["policy", "test"] => policy_test(Path::new(TEST_PATH)),
        ["policy", "test", path] => policy_test(Path::new(path)),
        ["check", path] => decide_file(path, false),
        ["explain", path] => decide_file(path, true),
        ["hook", "claude-code"] => serve(HostKind::ClaudeCode),
        ["hook", "codex"] => serve(HostKind::Codex),
        ["hook", "cursor"] => serve(HostKind::Cursor),
        ["log"] => report(show_log(20)),
        ["log", count] => {
            let count = count.parse().unwrap_or(20);
            report(show_log(count))
        }
        ["verify"] => report(verify()),
        ["doctor"] => report(Ok(doctor())),
        ["version" | "--version" | "-V"] => report(Ok(version())),
        ["task", rest @ ..] => report(task_command(rest)),
        _ => {
            usage();
            ExitCode::FAILURE
        }
    }
}

/// Writes the selected adapter's denial before a panic unwinds.
fn install_panic_hook(kind: HostKind) {
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(text) = respond(kind, Err("the hook stopped unexpectedly")) {
            println!("{text}");
        }
        eprintln!("shiin: internal error: {info}");
    }));
}

/// Holds an exclusive lock until the caller has finished updating workspace state.
///
/// Contention denies immediately rather than evaluating with stale counters or
/// waiting beyond the hook deadline. The operating system releases the lock when
/// the file closes, including when the process exits.
fn lock_workspace() -> Result<File, String> {
    fs::create_dir_all(".shiin").map_err(|error| format!(".shiin: {error}"))?;
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(LOCK_PATH)
        .map_err(|error| format!("{LOCK_PATH}: {error}"))?;
    file.try_lock()
        .map_err(|error| format!("{LOCK_PATH}: {error}"))?;
    Ok(file)
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
    eprintln!("  policy test [dir]         run policy cases from .shiin/tests");
    eprintln!("  check <request.json>      decide one action request");
    eprintln!("  explain <request.json>    decide, and show which stage decided");
    eprintln!("  hook claude-code          serve a Claude Code PreToolUse hook on stdin");
    eprintln!("  hook codex                serve an OpenAI Codex CLI hook on stdin");
    eprintln!("  hook cursor               serve a Cursor hook on stdin");
    eprintln!("  log [count]               show recent audit events");
    eprintln!("  verify                    check the audit log's hash chain");
    eprintln!("  doctor                    check this workspace's setup");
    eprintln!("  version                   print the version of this build");
    eprintln!();
    eprintln!("  task start <description> [options]   declare what the agent is working on");
    eprintln!("  task show                            show the active task and its usage");
    eprintln!("  task end                             end the active task");
    eprintln!();
    eprintln!("  task start options:");
    eprintln!("    --scope <glob>         limit changes to these paths, repeatable");
    eprintln!("    --forbid <action>      refuse these action types, repeatable");
    eprintln!("    --allow-host <host>    limit network effects to these hosts, repeatable");
    eprintln!("    --ask-for <action>     always ask a person for these action types, repeatable");
    eprintln!("    --max-writes <n>       most files the task may write or create");
    eprintln!("    --max-deletes <n>      most files the task may delete");
    eprintln!("    --max-commands <n>     most commands the task may run");
    eprintln!("    --minutes <n>          end the task after this many minutes");
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

/// Runs the policy cases in a directory.
///
/// A case has the same shape as the specification's test vectors, so the cases an
/// operator writes are checked by the same runner as the project's own corpus. A case
/// that carries no policy is evaluated against this workspace's policy, which is what
/// makes it a regression test for a policy change.
fn policy_test(directory: &Path) -> ExitCode {
    let outcome = (|| -> Result<Vec<String>, String> {
        if !directory.is_dir() {
            return Err(format!(
                "{} does not exist; put policy cases there, in the test-vector format",
                directory.display()
            ));
        }
        let root = workspace_root();
        let fallback = load_policy(Path::new(POLICY_PATH), &root)
            .ok()
            .map(|(policy, _)| policy);

        let mut paths = Vec::new();
        for entry in
            fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?
        {
            let path = entry.map_err(|error| error.to_string())?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                paths.push(path);
            }
        }
        paths.sort();
        if paths.is_empty() {
            return Err(format!("{} holds no cases", directory.display()));
        }

        let mut lines = Vec::new();
        let mut failed = 0_usize;
        for path in &paths {
            let text =
                fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
            let case: serde_json::Value = serde_json::from_str(&text)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            match vectors::run(&case, fallback.as_ref()) {
                Ok(result) if result.passed() => {
                    lines.push(format!("ok      {}  {}", result.id, result.actual.1));
                }
                Ok(result) => {
                    failed = failed.saturating_add(1);
                    lines.push(format!(
                        "FAILED  {}  expected {}/{}, got {}/{}",
                        result.id,
                        result.expected.0,
                        result.expected.1,
                        result.actual.0,
                        result.actual.1
                    ));
                }
                Err(error) => {
                    failed = failed.saturating_add(1);
                    lines.push(format!("ERROR   {}: {error}", path.display()));
                }
            }
        }
        lines.push(String::new());
        lines.push(format!("{} case(s), {failed} failed", paths.len()));
        if failed == 0 {
            Ok(lines)
        } else {
            Err(lines.join("\n"))
        }
    })();

    match outcome {
        Ok(lines) => {
            println!("{}", lines.join("\n"));
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// Reads the active task declaration, if there is one.
fn load_task() -> Result<Option<TaskDeclaration>, String> {
    let path = Path::new(TASK_PATH);
    if !path
        .try_exists()
        .map_err(|error| format!("{TASK_PATH}: {error}"))?
    {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|error| format!("{TASK_PATH}: {error}"))?;
    let task: TaskDeclaration = serde_json::from_str(&text)
        .map_err(|error| format!("SHIIN-GEN-0001: {TASK_PATH}: {error}"))?;
    Ok(Some(task))
}

/// Reads counters for an active task, refusing missing or damaged state.
///
/// Without a task there is no budget to load. An active task always has counters
/// written by `task start`; losing those counters must never reset its budget
/// ([ADR-0013](../../../docs/adr/0013-fail-closed-failure-semantics.md)).
fn load_usage(task: Option<&TaskDeclaration>) -> Result<TaskUsage, String> {
    if task.is_none() {
        return Ok(TaskUsage::default());
    }
    let text = fs::read_to_string(USAGE_PATH).map_err(|error| format!("{USAGE_PATH}: {error}"))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| format!("SHIIN-GEN-0001: {USAGE_PATH}: {error}"))?;
    for field in ["files_written", "files_deleted", "commands"] {
        if value
            .get(field)
            .and_then(serde_json::Value::as_u64)
            .is_none()
        {
            return Err(format!(
                "SHIIN-GEN-0001: {USAGE_PATH}: missing or invalid {field}"
            ));
        }
    }
    serde_json::from_value(value).map_err(|error| format!("SHIIN-GEN-0001: {USAGE_PATH}: {error}"))
}

/// Records the task's counters after an action was allowed.
fn save_usage(usage: TaskUsage) -> Result<(), String> {
    let text = serde_json::to_string(&usage).map_err(|error| format!("SHIIN-GEN-0001: {error}"))?;
    fs::write(USAGE_PATH, text).map_err(|error| format!("{USAGE_PATH}: {error}"))?;
    File::options()
        .write(true)
        .open(USAGE_PATH)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("{USAGE_PATH}: {error}"))
}

/// Handles `shiin task`.
fn task_command(arguments: &[&str]) -> Result<String, String> {
    let _lock = lock_workspace()?;
    match arguments {
        ["start", description, options @ ..] => start_task(description, options),
        ["show"] => show_task(),
        ["end"] => end_task(),
        _ => Err("usage: shiin task start <description> [options] | show | end".to_owned()),
    }
}

/// Declares a task, replacing any task already active.
///
/// A task is confirmed by the person who runs this command. An agent cannot declare one
/// for itself, which is the whole point of binding actions to intent
/// ([ADR-0020](../../../docs/adr/0020-structured-intent-binding.md)).
fn start_task(description: &str, options: &[&str]) -> Result<String, String> {
    let mut scope = Vec::new();
    let mut forbid = Vec::new();
    let mut hosts = Vec::new();
    let mut ask_for = Vec::new();
    let mut limits = Limits::default();
    let mut minutes: Option<u64> = None;

    let mut rest = options;
    while let [flag, value, tail @ ..] = rest {
        let number = || {
            value
                .parse::<u64>()
                .map_err(|_| format!("{flag} needs a whole number, not `{value}`"))
        };
        match *flag {
            "--scope" => scope.push((*value).to_owned()),
            "--forbid" => forbid.push((*value).to_owned()),
            "--allow-host" => hosts.push((*value).to_owned()),
            "--ask-for" => ask_for.push((*value).to_owned()),
            "--max-writes" => limits.files_written = Some(number()?),
            "--max-deletes" => limits.files_deleted = Some(number()?),
            "--max-commands" => limits.commands = Some(number()?),
            "--minutes" => minutes = Some(number()?),
            other => return Err(format!("unknown option `{other}`")),
        }
        rest = tail;
    }
    if let [leftover] = rest {
        return Err(format!("`{leftover}` has no value"));
    }

    let expires_at = minutes.map(|minutes| {
        clock::rfc3339_from_unix_seconds(
            clock::now_unix_seconds().saturating_add(minutes.saturating_mul(60)),
        )
    });
    let task = TaskDeclaration {
        id: clock::new_id(),
        description: Some(description.to_owned()),
        scope: Scope { paths: scope },
        forbid: shiin_schema::Forbid {
            action_types: forbid,
        },
        allow: shiin_schema::AllowHosts { hosts },
        limits,
        require_pending_for: ask_for,
        expires_at,
    };

    if let Some(parent) = Path::new(TASK_PATH).parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let text =
        serde_json::to_string_pretty(&task).map_err(|error| format!("SHIIN-GEN-0001: {error}"))?;
    fs::write(TASK_PATH, text).map_err(|error| format!("{TASK_PATH}: {error}"))?;
    save_usage(TaskUsage::default())?;

    let mut lines = vec![format!("task {} started: {description}", task.id)];
    if task.scope.paths.is_empty() {
        lines.push(
            "No scope was given, so changes are limited only by policy. Consider --scope."
                .to_owned(),
        );
    }
    lines.push("Only this task's constraints apply; policy still applies on top.".to_owned());
    Ok(lines.join("\n"))
}

/// Shows the active task, its constraints, and what it has used.
fn show_task() -> Result<String, String> {
    let Some(task) = load_task()? else {
        return Ok("no task is active".to_owned());
    };
    let usage = load_usage(Some(&task))?;
    let list = |values: &[String]| {
        if values.is_empty() {
            "none".to_owned()
        } else {
            values.join(", ")
        }
    };
    let limit = |value: Option<u64>| value.map_or_else(|| "none".to_owned(), |n| n.to_string());

    Ok(format!(
        "id            {}\ndescription   {}\nscope         {}\nforbidden     {}\nhosts         {}\nask for       {}\nexpires       {}\nwrites        {} of {}\ndeletes       {} of {}\ncommands      {} of {}",
        task.id,
        task.description.as_deref().unwrap_or(""),
        list(&task.scope.paths),
        list(&task.forbid.action_types),
        list(&task.allow.hosts),
        list(&task.require_pending_for),
        task.expires_at.as_deref().unwrap_or("never"),
        usage.files_written,
        limit(task.limits.files_written),
        usage.files_deleted,
        limit(task.limits.files_deleted),
        usage.commands,
        limit(task.limits.commands),
    ))
}

/// Ends the active task, so its constraints stop applying.
fn end_task() -> Result<String, String> {
    if !Path::new(TASK_PATH).exists() {
        return Ok("no task is active".to_owned());
    }
    fs::remove_file(TASK_PATH).map_err(|error| format!("{TASK_PATH}: {error}"))?;
    let _ = fs::remove_file(USAGE_PATH);
    Ok("task ended".to_owned())
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

/// The workspace this command applies to, canonicalized.
///
/// Canonicalizing matters: policy globs are resolved against this root and effects are
/// canonicalized, so a root reached through a symlink would put them in different
/// namespaces and nothing would match (`AR-008`).
fn workspace_root() -> String {
    let reported = std::env::var("SHIIN_WORKSPACE").unwrap_or_else(|_| {
        std::env::current_dir().map_or_else(
            |_| ".".to_owned(),
            |path| path.to_string_lossy().into_owned(),
        )
    });
    shiin_classify::canonical_root(&reported)
}

/// Decides a request read from a file, optionally explaining the outcome.
fn decide_file(path: &str, explain: bool) -> ExitCode {
    let outcome = (|| -> Result<Decision, String> {
        let _lock = lock_workspace()?;
        let text = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
        let request: ActionRequest =
            serde_json::from_str(&text).map_err(|error| format!("{path}: {error}"))?;
        let (source, compiled) = load_policy(Path::new(POLICY_PATH), &request.workspace.root)?;
        let task = load_task()?;
        Ok(decide(&Inputs {
            request: &request,
            policy: &compiled,
            source: &source,
            task: task.as_ref(),
            usage: load_usage(task.as_ref())?,
            now: &clock::now_rfc3339(),
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

/// Which host's protocol to speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostKind {
    /// Claude Code, through a `PreToolUse` hook.
    ClaudeCode,
    /// `OpenAI` Codex CLI, through a `PreToolUse` hook.
    Codex,
    /// Cursor, through one of its per-event hooks.
    Cursor,
}

/// Serves one hook call for a host.
///
/// Every failure prints a denial in that host's own protocol, because a host that hears
/// nothing allows the action. The exit code stays zero so the host reads the response as
/// a decision rather than as a broken hook.
fn serve(kind: HostKind) -> ExitCode {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut input = String::new();
        let outcome = std::io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| format!("cannot read the hook payload: {error}"))
            .and_then(|_| decide_for_host(kind, &input));
        let _ = sender.send(outcome);
    });
    let outcome = match receiver.recv_timeout(HOOK_TIMEOUT) {
        Ok(outcome) => outcome,
        Err(RecvTimeoutError::Timeout) => Err("the hook exceeded its internal deadline".to_owned()),
        // The panic hook already wrote one denial. Do not emit a second JSON response.
        Err(RecvTimeoutError::Disconnected) => return ExitCode::SUCCESS,
    };
    let text = match outcome {
        Ok(decision) => respond(kind, Ok(&decision)),
        Err(detail) => {
            eprintln!("shiin: {detail}");
            respond(kind, Err(detail.as_str()))
        }
    };
    match text {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        // Without a response the host would allow the action, so block it outright.
        Err(_) => ExitCode::from(2),
    }
}

/// Renders a decision, or a failure, in a host's own protocol.
fn respond(kind: HostKind, decision: Result<&Decision, &str>) -> Result<String, serde_json::Error> {
    match kind {
        HostKind::ClaudeCode | HostKind::Codex => {
            let response = match decision {
                Ok(decision) => HookResponse::from_decision(decision),
                Err(detail) => HookResponse::fail_closed(detail),
            };
            serde_json::to_string(&response)
        }
        HostKind::Cursor => {
            let response = match decision {
                Ok(decision) => cursor::HookResponse::from_decision(decision),
                Err(detail) => cursor::HookResponse::fail_closed(detail),
            };
            serde_json::to_string(&response)
        }
    }
}

/// Builds the action request for a host's payload, then decides it.
fn decide_for_host(kind: HostKind, input: &str) -> Result<Decision, String> {
    let _lock = lock_workspace()?;
    let fallback = workspace_root();
    let (mut request, root) = match kind {
        HostKind::ClaudeCode => {
            let payload: PreToolUse = serde_json::from_str(input)
                .map_err(|error| format!("the hook payload is not valid: {error}"))?;
            let root = if payload.cwd.is_empty() {
                fallback
            } else {
                shiin_classify::canonical_root(&payload.cwd)
            };
            (claude_code::to_action_request(&payload, &root), root)
        }
        HostKind::Codex => {
            let payload: codex::PreToolUse = serde_json::from_str(input)
                .map_err(|error| format!("the hook payload is not valid: {error}"))?;
            let root = if payload.cwd.is_empty() {
                fallback
            } else {
                shiin_classify::canonical_root(&payload.cwd)
            };
            (codex::to_action_request(&payload, &root), root)
        }
        HostKind::Cursor => {
            let payload: cursor::HookPayload = serde_json::from_str(input)
                .map_err(|error| format!("the hook payload is not valid: {error}"))?;
            let root = shiin_classify::canonical_root(&payload.workspace(&fallback));
            (cursor::to_action_request(&payload, &root), root)
        }
    };

    let (source, compiled) = load_policy(Path::new(POLICY_PATH), &root)?;
    request.policy_snapshot = source
        .layers
        .first()
        .map(|layer| layer.digest.clone())
        .unwrap_or_default();

    let task = load_task()?;
    request.task = task.as_ref().map(|task| task.id.clone());
    let usage = load_usage(task.as_ref())?;
    let decision = decide(&Inputs {
        request: &request,
        policy: &compiled,
        source: &source,
        task: task.as_ref(),
        usage,
        now: &clock::now_rfc3339(),
        mode: mode(),
    });

    // The decision is recorded before the action is allowed to proceed.
    record(&request, &decision)?;
    // A refused action never consumes the task's budget.
    if task.is_some() && decision.decision == DecisionValue::Allow {
        save_usage(usage.plus(shiin_engine::usage_of(&request)))?;
    }
    Ok(decision)
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

    match load_task() {
        Ok(Some(task)) => lines.push(format!(
            "task          active: {}",
            task.description.as_deref().unwrap_or(&task.id)
        )),
        Ok(None) => lines.push("task          none active, so only policy applies".to_owned()),
        Err(error) => lines.push(format!("task          PROBLEM: {error}")),
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

#[cfg(test)]
mod tests {
    use super::{HostKind, install_panic_hook};

    #[test]
    #[expect(clippy::panic, reason = "inject a panic in an isolated test process")]
    fn a_panic_emits_one_denial_in_each_adapter_protocol() {
        if let Ok(host) = std::env::var("SHIIN_TEST_PANIC_HOST") {
            let kind = match host.as_str() {
                "cursor" => HostKind::Cursor,
                "codex" => HostKind::Codex,
                _ => HostKind::ClaudeCode,
            };
            install_panic_hook(kind);
            panic!("EXAMPLE injected hook panic");
        }
        for host in ["claude-code", "codex", "cursor"] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "tests::a_panic_emits_one_denial_in_each_adapter_protocol",
                    "--nocapture",
                ])
                .env("SHIIN_TEST_PANIC_HOST", host)
                .output()
                .unwrap();
            assert!(!output.status.success(), "the injected panic must occur");
            let text = String::from_utf8_lossy(&output.stdout);
            let responses: Vec<serde_json::Value> = text
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            assert_eq!(responses.len(), 1, "{host}: {text}");
            let decision = if host == "cursor" {
                &responses[0]["permission"]
            } else {
                &responses[0]["hookSpecificOutput"]["permissionDecision"]
            };
            assert_eq!(decision, "deny", "{host}: {text}");
        }
    }
}
