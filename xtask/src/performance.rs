//! Repeatable latency measurements for the current daemonless implementation.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "performance is an operator-facing repository command"
)]

use std::fs;
use std::hint::black_box;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use shiin_engine::{Inputs, decide};
use shiin_policy::{CompiledPolicy, toml_front_end};
use shiin_schema::{ActionRequest, Mode, TaskUsage, clock};

const POLICY: &str = "version = 1\n[[rule]]\nid = 'read-source'\neffect = 'allow'\naction_types = ['fs.read']\nresources = ['./src/**']\n[risk]\nopaque_command = 'pending'\nsecret_path = 'deny'\n";

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
            eprintln!("performance: {error}");
            ExitCode::FAILURE
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "sequential measurements share one fixture and produce one report"
)]
fn measure() -> Result<bool, Box<dyn std::error::Error>> {
    let binary = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "target/release/shiin".to_owned());
    let binary = fs::canonicalize(binary)?;
    let workspace = Workspace(std::env::temp_dir().join(format!("shiin-perf-{}", clock::new_id())));
    fs::create_dir_all(workspace.0.join(".shiin"))?;
    fs::write(workspace.0.join(".shiin/policy.toml"), POLICY)?;
    let root = fs::canonicalize(&workspace.0)?
        .to_string_lossy()
        .into_owned();
    let mut request: ActionRequest = serde_json::from_value(
        serde_json::from_str::<Value>(include_str!(
            "../../docs/spec/test-vectors/tv-001-allowed-edit.v1.json"
        ))?["action_request"]
            .clone(),
    )?;
    "fs.read".clone_into(&mut request.action_type);
    request.workspace.root.clone_from(&root);
    for effect in &mut request.effects {
        effect.value = format!("{root}/src/login.rs");
    }
    let source = toml_front_end::parse(POLICY)?;
    let compiled = CompiledPolicy::compile(&source, &root)?;
    let inputs = Inputs {
        request: &request,
        policy: &compiled,
        source: &source,
        task: None,
        usage: TaskUsage::default(),
        now: "2026-10-15T00:00:00Z",
        mode: Mode::Unattended,
    };
    println!(
        "Performance sample: {} / {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!("| Path | Samples | Median ms | p95 ms | Max ms | Budget ms | Result |");
    println!("|---|---:|---:|---:|---:|---:|---|");
    let mut passed = true;
    let mut samples = Vec::new();
    for _ in 0..250 {
        let start = Instant::now();
        let decision = black_box(decide(black_box(&inputs)));
        samples.push(start.elapsed());
        if decision.decision != shiin_schema::DecisionValue::Allow {
            return Err("the benchmark request must actually be allowed".into());
        }
    }
    passed &= report("Engine evaluation", &mut samples, Duration::from_millis(50));
    samples.clear();
    for _ in 0..25 {
        let start = Instant::now();
        let source = toml_front_end::parse(black_box(POLICY))?;
        black_box(CompiledPolicy::compile(&source, &root)?);
        samples.push(start.elapsed());
    }
    passed &= report(
        "Policy parse and compilation",
        &mut samples,
        Duration::from_secs(1),
    );
    for host in ["claude-code", "codex", "cursor"] {
        let payload = if host == "cursor" {
            json!({"hook_event_name":"beforeReadFile", "conversation_id":"EXAMPLE-perf",
                "workspace_roots":[root], "file_path":"src/login.rs"})
        } else {
            json!({"session_id":"EXAMPLE-perf", "cwd":root, "tool_name":"Read",
                "tool_input":{"file_path":"src/login.rs"}})
        };
        samples.clear();
        for _ in 0..20 {
            let start = Instant::now();
            let mut child = Command::new(&binary)
                .args(["hook", host])
                .current_dir(&workspace.0)
                .env("SHIIN_UNATTENDED", "1")
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
            samples.push(start.elapsed());
            let response: Value = serde_json::from_slice(&output.stdout)?;
            let permission = if host == "cursor" {
                &response["permission"]
            } else {
                &response["hookSpecificOutput"]["permissionDecision"]
            };
            if !output.status.success() || permission != "allow" {
                return Err(format!(
                    "{host} returned a denial; refusing to benchmark a broken setup"
                )
                .into());
            }
        }
        passed &= report(
            &format!("{host} process, policy, and durable audit"),
            &mut samples,
            Duration::from_millis(200),
        );
    }
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines().filter(|line| line.starts_with("VmHWM:")) {
            println!("Harness process peak RSS: {line}");
        }
    }
    println!("Small-policy local sample; hook measurements include a growing log of 0–120 events.");
    println!(
        "Component memory budgets, large logs, other platforms, and live hosts are not established by this sample."
    );
    Ok(passed)
}

fn report(label: &str, samples: &mut [Duration], budget: Duration) -> bool {
    samples.sort_unstable();
    let max = samples.last().copied().unwrap_or_default();
    let median = samples.get(samples.len() / 2).copied().unwrap_or_default();
    let p95 = samples
        .get(samples.len().saturating_mul(95) / 100)
        .copied()
        .unwrap_or_default();
    let passed = max <= budget;
    println!(
        "| {label} | {} | {:.3} | {:.3} | {:.3} | {} | {} |",
        samples.len(),
        median.as_secs_f64() * 1000.0,
        p95.as_secs_f64() * 1000.0,
        max.as_secs_f64() * 1000.0,
        budget.as_millis(),
        if passed { "pass" } else { "FAIL" }
    );
    passed
}
