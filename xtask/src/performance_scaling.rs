//! Checks the existing latency budgets with larger policies and a growing audit log.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "performance commands report measurements"
)]

use serde_json::{Value, json};
use shiin_engine::{Inputs, decide};
use shiin_policy::{CompiledPolicy, toml_front_end};
use shiin_schema::{ActionRequest, DecisionValue, Mode, TaskUsage, clock};
use std::{
    fs,
    hint::black_box,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};

struct Workspace(PathBuf);

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn run() -> ExitCode {
    match measure() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("performance-scaling: {error}");
            ExitCode::FAILURE
        }
    }
}

fn fixture() -> Result<(Workspace, ActionRequest), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("ss-{}", clock::new_id()));
    fs::create_dir_all(root.join(".shiin"))?;
    let workspace = Workspace(fs::canonicalize(root)?);
    let mut action: ActionRequest = serde_json::from_value(
        serde_json::from_str::<Value>(include_str!(
            "../../docs/spec/test-vectors/tv-001-allowed-edit.v1.json"
        ))?["action_request"]
            .clone(),
    )?;
    "fs.read".clone_into(&mut action.action_type);
    workspace
        .0
        .to_str()
        .ok_or("workspace must be UTF-8")?
        .clone_into(&mut action.workspace.root);
    for effect in &mut action.effects {
        effect.value = format!("{}/src/EXAMPLE.rs", workspace.0.display());
    }
    Ok((workspace, action))
}

pub(super) fn policy(rules: usize) -> String {
    let mut source = "version = 1\n[capabilities.EXAMPLE]\nid = 'EXAMPLE'\nallow = true\naction_types = ['*']\n[[rule]]\nid = 'source'\neffect = 'allow'\naction_types = ['fs.read']\nresources = ['./src/**']\n".to_owned();
    for index in 1..rules {
        use std::fmt::Write as _;
        let _ = writeln!(
            source,
            "[[rule]]\nid = 'EXAMPLE-{index}'\neffect = 'allow'\naction_types = ['fs.read']\nresources = ['./unused-EXAMPLE-{index}/**']"
        );
    }
    source
}

fn measure() -> Result<bool, Box<dyn std::error::Error>> {
    let binary = fs::canonicalize(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "target/release/shiin".to_owned()),
    )?;
    let (workspace, action) = fixture()?;
    println!(
        "Scaling sample: {} / {}; existing latency budgets",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!("| Path | Samples | Median ms | p95 ms | Max ms | Budget ms | Result |");
    println!("|---|---:|---:|---:|---:|---:|---|");
    let mut passed = true;
    for count in [100, 500] {
        let text = policy(count);
        let source = toml_front_end::parse(&text)?;
        let compiled = CompiledPolicy::compile(&source, &action.workspace.root)?;
        let inputs = Inputs {
            request: &action,
            policy: &compiled,
            source: &source,
            task: None,
            usage: TaskUsage::default(),
            now: "2026-10-16T00:00:00Z",
            mode: Mode::Unattended,
        };
        let mut samples = Vec::new();
        for _ in 0..25 {
            let start = Instant::now();
            let decision = black_box(decide(black_box(&inputs)));
            samples.push(start.elapsed());
            if decision.decision != DecisionValue::Allow {
                return Err("scaling fixture must actually be allowed".into());
            }
        }
        passed &= super::performance::report(
            &format!("Engine, {count} rules"),
            &mut samples,
            Duration::from_millis(50),
        );
        samples.clear();
        for _ in 0..5 {
            let start = Instant::now();
            let source = toml_front_end::parse(black_box(&text))?;
            black_box(CompiledPolicy::compile(&source, &action.workspace.root)?);
            samples.push(start.elapsed());
        }
        passed &= super::performance::report(
            &format!("Parse/compile, {count} rules"),
            &mut samples,
            Duration::from_secs(1),
        );
    }
    fs::write(workspace.0.join(".shiin/policy.toml"), policy(100))?;
    let log = shiin_audit::AuditLog::new(workspace.0.join(".shiin/audit.jsonl"));
    for _ in 0..1000 {
        log.append(
            "decision.made",
            "2026-10-16T00:00:00Z",
            &json!({"decision":"deny","id":"EXAMPLE-seed"}),
        )?;
    }
    for host in ["claude-code", "codex", "cursor"] {
        let mut samples = Vec::new();
        for _ in 0..5 {
            samples.push(hook_sample(&binary, &workspace.0, host)?);
        }
        passed &= super::performance::report(
            &format!("{host}, 100 rules and 1000+ events"),
            &mut samples,
            Duration::from_millis(200),
        );
    }
    println!(
        "Bounded scaling sample; component memory, larger workloads, and live host behavior remain unmeasured."
    );
    Ok(passed)
}

fn hook_sample(
    binary: &Path,
    root: &Path,
    host: &str,
) -> Result<Duration, Box<dyn std::error::Error>> {
    let payload = if host == "cursor" {
        json!({"hook_event_name":"beforeReadFile","conversation_id":"EXAMPLE-scale","workspace_roots":[root],"file_path":"src/EXAMPLE.rs"})
    } else {
        json!({"session_id":"EXAMPLE-scale","cwd":root,"tool_name":"Read","tool_input":{"file_path":"src/EXAMPLE.rs"}})
    };
    let start = Instant::now();
    let mut child = Command::new(binary)
        .args(["hook", host])
        .current_dir(root)
        .env("SHIIN_UNATTENDED", "1")
        .env_remove("SHIIN_SOCKET")
        .env_remove("SHIIN_DAEMON")
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
    let elapsed = start.elapsed();
    let result: Value = serde_json::from_slice(&output.stdout)?;
    let permission = if host == "cursor" {
        &result["permission"]
    } else {
        &result["hookSpecificOutput"]["permissionDecision"]
    };
    if !output.status.success() || permission != "allow" {
        return Err(format!("{host} scaling fixture was refused").into());
    }
    Ok(elapsed)
}
