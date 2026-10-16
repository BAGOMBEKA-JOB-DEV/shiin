//! Reproducible intent measurements over the published, labelled task corpus.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "repository measurement commands report their results"
)]

use serde_json::Value;
use shiin_engine::vectors;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

pub(crate) fn run() -> ExitCode {
    match measure() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("intent-evaluation: {error}");
            ExitCode::FAILURE
        }
    }
}

fn measure() -> Result<bool, Box<dyn std::error::Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/spec/test-vectors");
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    let mut total = 0;
    let mut matched = 0;
    let mut changed = 0;
    let mut false_allows = 0;
    let mut false_denies = 0;
    let mut labelled_allows = 0;
    let mut labelled_nonallows = 0;
    println!(
        "Intent evaluation: labelled specification corpus, engine {}",
        shiin_engine::ENGINE_VERSION
    );
    println!("| Case | Policy without task | With task | Expected | Result |");
    println!("|---|---|---|---|---|");
    for path in paths {
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let case: Value = serde_json::from_slice(&fs::read(path)?)?;
        if !case.get("task").is_some_and(Value::is_object) {
            continue;
        }
        let constrained = vectors::run(&case, None)?;
        let mut baseline = case;
        baseline["task"] = Value::Null;
        baseline["action_request"]["task"] = Value::Null;
        let baseline = vectors::run(&baseline, None)?;
        total += 1;
        matched += usize::from(constrained.passed());
        changed += usize::from(baseline.actual != constrained.actual);
        labelled_allows += usize::from(constrained.expected.0 == "allow");
        labelled_nonallows += usize::from(constrained.expected.0 != "allow");
        false_allows +=
            usize::from(constrained.actual.0 == "allow" && constrained.expected.0 != "allow");
        false_denies +=
            usize::from(constrained.actual.0 == "deny" && constrained.expected.0 == "allow");
        println!(
            "| {} | {} | {} | {} | {} |",
            constrained.id,
            baseline.actual.1,
            constrained.actual.1,
            constrained.expected.1,
            if constrained.passed() { "pass" } else { "FAIL" }
        );
    }
    if total == 0 {
        return Err("the intent measurement corpus contains no task cases".into());
    }
    println!(
        "Exact decision/reason matches: {matched}/{total}; task-dependent outcome changes: {changed}."
    );
    println!(
        "Corpus-labelled false allows: {false_allows}/{labelled_nonallows}; false denies: {false_denies}/{labelled_allows}."
    );
    println!(
        "This small specification corpus does not estimate real-world false-positive or false-negative rates, host interception, or usability."
    );
    Ok(matched == total && false_allows == 0 && false_denies == 0)
}
