//! Scheduled network link checks and an explicitly opted-in bot issue report.

#![expect(
    clippy::print_stderr,
    reason = "repository checks report their failures"
)]

use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, ExitCode},
};

const TITLE: &str = "External documentation link check";

pub(crate) fn run() -> ExitCode {
    match check() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("link-health: {error}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<bool, Box<dyn std::error::Error>> {
    let report_issue = std::env::args().any(|argument| argument == "--report-issue");
    if report_issue && std::env::var("CI").as_deref() != Ok("true") {
        return Err("issue reporting requires explicit CI execution".into());
    }
    let directory = Path::new("target/link-health");
    fs::create_dir_all(directory)?;
    let report = directory.join("report.md");
    let status = Command::new("lychee")
        .args([
            "--config",
            "lychee.toml",
            "--offline=false",
            "--exclude-all-private=true",
            "--timeout",
            "15",
            "--max-retries",
            "1",
            "--max-concurrency",
            "8",
            "--exclude",
            r"https?://([^/]*\.)?example\.(com|org|net)(/|$)|https?://[^/]*\.example(/|$)",
            "--format",
            "markdown",
            "--output",
        ])
        .arg(&report)
        .args(["--", "./*.md", "docs"])
        .status()?;
    if report_issue {
        report_result(status.success(), &report)?;
    }
    Ok(status.success())
}

fn report_result(passed: bool, report: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let repository = std::env::var("GITHUB_REPOSITORY")?;
    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--repo",
            &repository,
            "--state",
            "open",
            "--author",
            "github-actions[bot]",
            "--search",
            "\"External documentation link check\" in:title",
            "--json",
            "number,title",
            "--limit",
            "100",
        ])
        .output()?;
    if !output.status.success() {
        return Err("could not inspect existing bot link report".into());
    }
    let issues: Vec<Value> = serde_json::from_slice(&output.stdout)?;
    let existing = issues
        .iter()
        .find(|issue| issue["title"] == TITLE)
        .and_then(|issue| issue["number"].as_u64());
    let mut command = Command::new("gh");
    match (passed, existing) {
        (true, None) => return Ok(()),
        (true, Some(number)) => {
            command.args([
                "issue",
                "close",
                &number.to_string(),
                "--repo",
                &repository,
                "--comment",
                "The scheduled external link check now passes.",
            ]);
        }
        (false, Some(number)) => {
            command
                .args([
                    "issue",
                    "comment",
                    &number.to_string(),
                    "--repo",
                    &repository,
                    "--body-file",
                ])
                .arg(report);
        }
        (false, None) => {
            command
                .args([
                    "issue",
                    "create",
                    "--repo",
                    &repository,
                    "--title",
                    TITLE,
                    "--body-file",
                ])
                .arg(report);
        }
    }
    if !command.status()?.success() {
        return Err("could not publish link-check result".into());
    }
    Ok(())
}
