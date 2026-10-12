//! Runs the specification's test vectors against the engine.
//!
//! This is the conformance bar from `docs/spec/test-vectors/README.md`: every vector
//! must reproduce its expected decision and reason code. It replaces the ADR-0014
//! spike, which proved the approach before these crates existed.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use shiin_engine::{Inputs, decide};
use shiin_policy::CompiledPolicy;
use shiin_schema::{ActionRequest, Mode, Policy, TaskDeclaration};

/// Evaluates one vector, returning the decision and reason it produced.
fn run(vector: &Value) -> Result<(String, String), String> {
    let request: ActionRequest = serde_json::from_value(vector["action_request"].clone())
        .map_err(|error| format!("the action request does not parse: {error}"))?;
    let source: Policy = serde_json::from_value(vector["policy"].clone())
        .map_err(|error| format!("the policy does not parse: {error}"))?;
    let task: Option<TaskDeclaration> = match vector.get("task") {
        Some(value) if value.is_object() => Some(
            serde_json::from_value(value.clone())
                .map_err(|error| format!("the task does not parse: {error}"))?,
        ),
        _ => None,
    };
    let mode = if vector["mode"].as_str() == Some("unattended") {
        Mode::Unattended
    } else {
        Mode::Attended
    };

    let compiled = CompiledPolicy::compile(&source, &request.workspace.root)
        .map_err(|error| format!("the policy does not compile: {error}"))?;
    let decision = decide(&Inputs {
        request: &request,
        policy: &compiled,
        source: &source,
        task: task.as_ref(),
        mode,
    });
    let value = serde_json::to_value(decision.decision)
        .map_err(|error| format!("the decision does not serialize: {error}"))?;
    let name = value
        .as_str()
        .ok_or_else(|| "a decision value is a string".to_owned())?;
    Ok((name.to_owned(), decision.reason))
}

/// Reads every vector file, newest specification format only.
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
    for (path, vector) in &loaded {
        let id = vector["id"].as_str().unwrap_or("?");
        let expected = (
            vector["expected"]["decision"].as_str().unwrap_or("?"),
            vector["expected"]["reason"].as_str().unwrap_or("?"),
        );
        match run(vector) {
            Ok((decision, reason)) => {
                if decision != expected.0 || reason != expected.1 {
                    failures.push(format!(
                        "{id}: expected {}/{}, got {decision}/{reason}",
                        expected.0, expected.1
                    ));
                }
            }
            Err(error) => failures.push(format!("{id} ({}): {error}", path.display())),
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
