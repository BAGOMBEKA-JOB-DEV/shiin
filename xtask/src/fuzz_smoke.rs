//! Seeds a bounded fuzz run from the published corpus on a dated nightly.

#![expect(
    clippy::print_stderr,
    reason = "repository checks report their failures"
)]

use std::{
    fs,
    path::Path,
    process::{Command, ExitCode},
};

const NIGHTLY: &str = "nightly-2026-10-15";

pub(crate) fn run() -> ExitCode {
    match exercise() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("fuzz-smoke: {error}");
            ExitCode::FAILURE
        }
    }
}

fn exercise() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let requests = root.join("fuzz/corpus/action-request");
    fs::create_dir_all(&requests)?;
    for entry in fs::read_dir(root.join("docs/spec/test-vectors"))? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let case: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        let filename = path.file_name().ok_or("vector filename missing")?;
        fs::write(
            requests.join(filename),
            serde_json::to_vec(&case["action_request"])?,
        )?;
    }
    let classification = root.join("fuzz/corpus/classification");
    fs::create_dir_all(&classification)?;
    for (index, seed) in [
        "cat 'EXAMPLE file'",
        "SELECT EXAMPLE FROM records",
        "SELECT 'EXAMPLE'; DROP TABLE records",
        "EXPLAIN ANALYZE SELECT EXAMPLE FROM records",
        "echo $(EXAMPLE)",
    ]
    .iter()
    .enumerate()
    {
        fs::write(classification.join(format!("seed-{index}")), seed)?;
    }
    for target in ["action-request", "classification"] {
        let status = Command::new("rustup")
            .current_dir(&root)
            .args([
                "run",
                NIGHTLY,
                "cargo",
                "fuzz",
                "run",
                target,
                "--",
                "-max_total_time=20",
                "-max_len=65536",
                "-rss_limit_mb=1024",
            ])
            .status()?;
        if !status.success() {
            return Err(format!("{target} fuzz smoke failed").into());
        }
    }
    Ok(())
}
