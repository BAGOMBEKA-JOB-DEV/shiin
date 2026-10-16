//! Measures component heap allocations without instrumenting product binaries.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "memory measurements report their results"
)]

use allocation_counter::{AllocationInfo, measure};
use shiin_engine::{Inputs, decide};
use shiin_policy::{CompiledPolicy, toml_front_end};
use shiin_schema::{ActionRequest, DecisionValue, Mode, TaskUsage};
use std::{hint::black_box, process::ExitCode};

const ROOT: &str = "/shiin-EXAMPLE-memory-root/project";

pub(crate) fn run() -> ExitCode {
    match check() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("memory: {error}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<bool, Box<dyn std::error::Error>> {
    let calibration = measure(|| {
        black_box(vec![0_u8; 65_536]);
    });
    if calibration.bytes_max < 65_536 || calibration.bytes_current != 0 {
        return Err("heap counter did not observe the calibration allocation and drop".into());
    }
    let case: serde_json::Value = serde_json::from_str(include_str!(
        "../../docs/spec/test-vectors/tv-001-allowed-edit.v1.json"
    ))?;
    let mut request: ActionRequest = serde_json::from_value(case["action_request"].clone())?;
    "fs.read".clone_into(&mut request.action_type);
    ROOT.clone_into(&mut request.workspace.root);
    for effect in &mut request.effects {
        effect.value = format!("{ROOT}/src/EXAMPLE.rs");
    }
    println!(
        "Component heap sample: {} / {}; decimal MB; current thread only",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!(
        "| Component | Rules | Peak heap bytes | Total allocated bytes | Budget bytes | Result |"
    );
    println!("|---|---:|---:|---:|---:|---|");
    let mut passed = true;
    for count in [1, 100, 500] {
        let text = super::performance_scaling::policy(count);
        let mut compilation = None;
        let stats = measure(|| {
            compilation = Some(toml_front_end::parse(black_box(&text)).and_then(|source| {
                CompiledPolicy::compile(&source, ROOT).map(|compiled| (source, compiled))
            }));
        });
        let (source, compiled) = compilation.ok_or("compilation did not run")??;
        passed &= report("Policy parse/compile", count, &stats, 100_000_000);
        let inputs = Inputs {
            request: &request,
            policy: &compiled,
            source: &source,
            task: None,
            usage: TaskUsage::default(),
            now: "2026-10-16T00:00:00Z",
            mode: Mode::Unattended,
        };
        // Each measurement includes decision creation and drop, with fixtures already resident.
        let mut worst = AllocationInfo::default();
        for _ in 0..25 {
            let mut allowed = false;
            let stats = measure(|| {
                allowed = black_box(decide(black_box(&inputs))).decision == DecisionValue::Allow;
            });
            if !allowed {
                return Err("memory fixture must actually be allowed".into());
            }
            if stats.bytes_max > worst.bytes_max {
                worst = stats;
            }
        }
        passed &= report("Engine evaluation (25 samples)", count, &worst, 10_000_000);
    }
    println!(
        "Heap deltas exclude pre-existing fixtures, stack, allocator overhead, native allocations, and other threads; they do not establish process RSS budgets."
    );
    Ok(passed)
}

fn report(component: &str, rules: usize, stats: &AllocationInfo, budget: u64) -> bool {
    let passed = stats.bytes_max <= budget;
    println!(
        "| {component} | {rules} | {} | {} | {budget} | {} |",
        stats.bytes_max,
        stats.bytes_total,
        if passed { "pass" } else { "FAIL" }
    );
    passed
}
