//! Exercises the executables extracted from the actual release archives.

#![expect(
    clippy::print_stderr,
    clippy::print_stdout,
    reason = "release qualification reports its results"
)]

use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use std::{
    fs,
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Output, Stdio},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn run() -> ExitCode {
    match check() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("release-smoke: {error}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<(), Box<dyn std::error::Error>> {
    let artifacts = PathBuf::from(
        std::env::args()
            .nth(2)
            .ok_or("provide the release artifact directory")?,
    );
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "shiin-EXAMPLE-release-{}",
        shiin_schema::clock::new_id()
    )));
    fs::create_dir_all(&scratch.0)?;
    let mut cli_archives = 0;
    for (index, item) in fs::read_dir(artifacts)?.enumerate() {
        let path = item?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("non-UTF-8 artifact name")?;
        let cli = name.starts_with("shiin-cli-");
        if !(cli || name.starts_with("shiind-"))
            || !(name.ends_with(".tar.xz") || path.extension().is_some_and(|ext| ext == "zip"))
        {
            continue;
        }
        verify_checksum(&path)?;
        let extracted = scratch.0.join(format!("archive-{index}"));
        fs::create_dir(&extracted)?;
        let mut extract = Command::new("tar");
        extract
            .arg("-xf")
            .arg(fs::canonicalize(&path)?)
            .arg("-C")
            .arg(&extracted);
        if !bounded(extract, None)?.status.success() {
            return Err("release archive could not be extracted".into());
        }
        let executable = if cli { "shiin" } else { "shiind" };
        let executable = format!("{executable}{}", std::env::consts::EXE_SUFFIX);
        let binaries: Vec<_> = walkdir::WalkDir::new(&extracted)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file() && entry.file_name() == executable.as_str())
            .map(walkdir::DirEntry::into_path)
            .collect();
        if binaries.len() != 1 {
            return Err("archive must contain exactly one expected executable".into());
        }
        let binary = fs::canonicalize(&binaries[0])?;
        if !binary.starts_with(fs::canonicalize(&extracted)?) {
            return Err("archive executable escaped extraction directory".into());
        }
        let mut version = Command::new(&binary);
        version.arg("--version");
        let output = bounded(version, None)?;
        let product_version = shiin_engine::ENGINE_VERSION
            .split('+')
            .next()
            .ok_or("missing product version")?;
        if !output.status.success()
            || !String::from_utf8_lossy(&output.stdout).contains(product_version)
        {
            return Err("release executable reported an unexpected version".into());
        }
        if cli {
            check_cli(&binary, &scratch.0.join(format!("workspace-{index}")))?;
            cli_archives += 1;
        }
        println!(
            "Verified archive checksum, extraction, version{}: {name}",
            if cli {
                ", hook decisions, and audit chain"
            } else {
                ""
            }
        );
    }
    if cli_archives == 0 {
        return Err("no CLI release archive was tested".into());
    }
    Ok(())
}

fn verify_checksum(archive: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let checksum = fs::read_to_string(format!("{}.sha256", archive.display()))?;
    let expected = checksum
        .split_whitespace()
        .next()
        .ok_or("missing archive checksum")?;
    let mut digest = Sha256::new();
    let mut file = fs::File::open(archive)?;
    let mut buffer = [0_u8; 8_192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if format!("{:x}", digest.finalize()) != expected {
        return Err("release archive checksum mismatch".into());
    }
    Ok(())
}

fn check_cli(binary: &Path, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(directory.join(".shiin"))?;
    fs::create_dir(directory.join("src"))?;
    fs::write(directory.join("src/EXAMPLE.txt"), "EXAMPLE fixture")?;
    fs::write(
        directory.join(".shiin/policy.toml"),
        "version=1\n[capabilities.EXAMPLE]\nid='EXAMPLE'\nallow=true\naction_types=['fs.read']\n[[rule]]\nid='EXAMPLE-read'\neffect='allow'\naction_types=['fs.read']\nresources=['./src/**']\n",
    )?;
    let root = fs::canonicalize(directory)?;
    let mut validate = Command::new(binary);
    validate.args(["policy", "validate"]).current_dir(&root);
    if !bounded(validate, None)?.status.success() {
        return Err("installed CLI cannot validate a policy".into());
    }
    for host in ["claude-code", "codex", "cursor"] {
        for (path, expected) in [("src/EXAMPLE.txt", "allow"), (".shiin/policy.toml", "deny")] {
            let payload = if host == "cursor" {
                json!({"hook_event_name":"beforeReadFile", "conversation_id":"EXAMPLE-release", "workspace_roots":[root], "file_path":path})
            } else {
                json!({"session_id":"EXAMPLE-release", "cwd":root, "tool_name":"Read", "tool_input":{"file_path":path}})
            };
            let mut hook = Command::new(binary);
            hook.args(["hook", host]).current_dir(&root);
            let output = bounded(hook, Some(payload.to_string().as_bytes()))?;
            let response: Value = serde_json::from_slice(&output.stdout)?;
            let decision = if host == "cursor" {
                &response["permission"]
            } else {
                &response["hookSpecificOutput"]["permissionDecision"]
            };
            if decision != expected {
                return Err(format!("archived {host} hook did not return {expected}").into());
            }
        }
    }
    let mut verify = Command::new(binary);
    verify.arg("verify").current_dir(root);
    if !bounded(verify, None)?.status.success() {
        return Err("archived CLI audit verification failed".into());
    }
    Ok(())
}

fn bounded(
    mut command: Command,
    input: Option<&[u8]>,
) -> Result<Output, Box<dyn std::error::Error>> {
    for variable in [
        "SHIIN_SOCKET",
        "SHIIN_DAEMON",
        "SHIIN_WORKSPACE",
        "SHIIN_UNATTENDED",
    ] {
        command.env_remove(variable);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(input) = input {
        child
            .stdin
            .take()
            .ok_or("child stdin unavailable")?
            .write_all(input)?;
    }
    drop(child.stdin.take());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(child.wait_with_output()?);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("release command exceeded ten-second deadline".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
