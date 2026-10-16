//! Publishes a reproducible core corpus and machine-readable evaluation report.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "repository checks report their artifacts"
)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use shiin_engine::vectors;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

pub(crate) fn run() -> ExitCode {
    match publish() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("conformance: {error}");
            ExitCode::FAILURE
        }
    }
}

fn files(directory: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    Ok(paths)
}

fn publish() -> Result<bool, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let output = std::env::args_os()
        .nth(2)
        .map_or_else(|| root.join("target/conformance"), PathBuf::from);
    fs::create_dir_all(output.join("vectors"))?;
    fs::create_dir_all(output.join("schemas"))?;
    let mut outcomes = Vec::new();
    let mut manifest = Vec::new();
    let mut passed = true;
    for path in files(&root.join("docs/spec/test-vectors"))? {
        let bytes = fs::read(&path)?;
        let case: Value = serde_json::from_slice(&bytes)?;
        let result = vectors::run(&case, None)?;
        passed &= result.passed();
        outcomes.push(json!({"id":result.id,"expected":{"decision":result.expected.0,"reason":result.expected.1},
            "actual":{"decision":result.actual.0,"reason":result.actual.1},"passed":result.passed()}));
        copy(&path, &bytes, "vectors", &output, &mut manifest)?;
    }
    if outcomes.is_empty() {
        return Err("the conformance corpus is empty".into());
    }
    for path in files(&root.join("docs/spec/schemas"))? {
        copy(&path, &fs::read(&path)?, "schemas", &output, &mut manifest)?;
    }
    let commit = Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !commit.status.success() {
        return Err("could not identify the corpus revision".into());
    }
    let commit = String::from_utf8(commit.stdout)?.trim().to_owned();
    let report = json!({"format":"shiin-core-conformance-report/1","commit":commit,
        "engine_version":shiin_engine::ENGINE_VERSION,"passed":passed,
        "scope":"core engine vectors; host interception, external MCP, identity and independent security review are not established",
        "cases":outcomes,"files":manifest});
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(
        output.join("README.txt"),
        "Shiin core conformance bundle\nreport.json identifies the tested revision, exact decision/reason results, and SHA-256 digests of the corpus and schemas.\nOnly files listed in the report belong to this bundle.\nThis unsigned report is not an independent audit, a live-host test, or proof of external MCP conformance.\n",
    )?;
    println!(
        "conformance: {} cases; {}; report: {}",
        report["cases"].as_array().map_or(0, Vec::len),
        if passed { "pass" } else { "FAIL" },
        output.join("report.json").display()
    );
    Ok(passed)
}

fn copy(
    path: &Path,
    bytes: &[u8],
    directory: &str,
    output: &Path,
    manifest: &mut Vec<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("corpus filename must be UTF-8")?;
    let relative = format!("{directory}/{name}");
    fs::write(output.join(&relative), bytes)?;
    manifest.push(json!({"path":relative,"sha256":format!("{:x}",Sha256::digest(bytes))}));
    Ok(())
}
