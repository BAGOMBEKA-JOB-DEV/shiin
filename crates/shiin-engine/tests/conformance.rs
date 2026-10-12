//! Runs the specification's test vectors against the engine.
//!
//! This is the conformance bar from `docs/spec/test-vectors/README.md`: every vector must
//! reproduce its expected decision and reason code. It uses the same runner as
//! `shiin policy test`, so an operator's cases are checked the way these are.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use shiin_engine::vectors;

/// Reads every vector file in the corpus.
fn vectors() -> Result<Vec<(PathBuf, Value)>, String> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/test-vectors");
    let entries =
        fs::read_dir(&directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    paths.sort();

    let mut loaded = Vec::new();
    for path in paths {
        let text =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        loaded.push((path, value));
    }
    Ok(loaded)
}

#[test]
fn every_test_vector_reproduces_its_expected_decision() -> Result<(), String> {
    let loaded = vectors()?;
    assert!(!loaded.is_empty(), "the conformance corpus is not empty");

    let mut failures = Vec::new();
    for (path, case) in &loaded {
        match vectors::run(case, None) {
            Ok(outcome) if outcome.passed() => {}
            Ok(outcome) => failures.push(format!(
                "{}: expected {}/{}, got {}/{}",
                outcome.id,
                outcome.expected.0,
                outcome.expected.1,
                outcome.actual.0,
                outcome.actual.1
            )),
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} of {} vectors did not reproduce:\n{}",
            failures.len(),
            loaded.len(),
            failures.join("\n")
        ))
    }
}
