//! The wrapper runs a child only after a durable allow and preserves its argv and status.

#![cfg(unix)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const ALLOW: &str = "version = 1\n[capabilities.EXAMPLE]\nid = 'EXAMPLE'\nallow = true\naction_types = ['*']\n[[rule]]\nid = 'EXAMPLE-command'\neffect = 'allow'\naction_types = ['shell.exec']\n";
const LITERAL: &str = "EXAMPLE spaces; 'quotes' $literal";

struct Workspace(PathBuf);

impl Workspace {
    fn new(policy: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let root =
            std::env::temp_dir().join(format!("shiin-exec-{}", shiin_schema::clock::new_id()));
        fs::create_dir_all(root.join(".shiin"))?;
        fs::write(root.join(".shiin/policy.toml"), policy)?;
        Ok(Self(root))
    }

    fn wrapper(&self) -> Result<Command, Box<dyn std::error::Error>> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_shiin"));
        command
            .args(["exec", "--session", "EXAMPLE-wrapper-session", "--"])
            .arg(std::env::current_exe()?)
            .args([
                "--exact",
                "wrapped_child_marker",
                "--nocapture",
                "--skip",
                LITERAL,
            ])
            .current_dir(&self.0)
            .env_remove("CI")
            .env_remove("SHIIN_UNATTENDED")
            .env_remove("SHIIN_DAEMON")
            .env_remove("SHIIN_SOCKET")
            .env("SHIIN_EXEC_TEST_MARKER", self.0.join("marker.json"));
        Ok(command)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn wrapped_child_marker() -> Result<(), Box<dyn std::error::Error>> {
    let Some(marker) = std::env::var_os("SHIIN_EXEC_TEST_MARKER") else {
        return Ok(());
    };
    let log = fs::read_to_string(".shiin/audit.jsonl")?;
    let last: serde_json::Value =
        serde_json::from_str(log.lines().last().ok_or("allow was not audited")?)?;
    assert_eq!(last["kind"], "decision.made");
    assert_eq!(last["payload"]["decision"], "allow");
    fs::write(
        marker,
        serde_json::to_vec(&std::env::args().collect::<Vec<_>>())?,
    )?;
    if let Ok(code) = std::env::var("SHIIN_EXEC_TEST_EXIT") {
        std::process::exit(code.parse()?);
    }
    Ok(())
}

#[test]
fn allow_is_durable_and_arguments_and_exit_status_are_preserved()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = Workspace::new(ALLOW)?;
    let output = workspace.wrapper()?.output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arguments: Vec<String> =
        serde_json::from_slice(&fs::read(workspace.0.join("marker.json"))?)?;
    assert!(arguments.iter().any(|argument| argument == LITERAL));
    assert!(!fs::read_to_string(workspace.0.join(".shiin/audit.jsonl"))?.contains(LITERAL));
    let output = workspace
        .wrapper()?
        .env("SHIIN_EXEC_TEST_EXIT", "7")
        .output()?;
    assert_eq!(output.status.code(), Some(7));
    Ok(())
}

#[test]
fn nonallow_and_failed_authorization_never_start_a_child() -> Result<(), Box<dyn std::error::Error>>
{
    for (policy, expected) in [
        (
            "version = 1\n[capabilities.EXAMPLE]\nid = 'EXAMPLE'\nallow = true\naction_types = ['*']\n[defaults]\ndecision = 'deny'\n",
            1,
        ),
        (
            "version = 1\n[capabilities.EXAMPLE]\nid = 'EXAMPLE'\nallow = true\naction_types = ['*']\n[defaults]\ndecision = 'pending'\n",
            2,
        ),
        ("invalid policy", 1),
    ] {
        let workspace = Workspace::new(policy)?;
        let output = workspace.wrapper()?.output()?;
        assert_eq!(output.status.code(), Some(expected));
        assert!(!workspace.0.join("marker.json").exists());
    }
    let workspace = Workspace::new(ALLOW)?;
    fs::write(
        workspace.0.join(".shiin/audit.jsonl"),
        "EXAMPLE-corrupted-log",
    )?;
    let output = workspace.wrapper()?.output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(!workspace.0.join("marker.json").exists());
    let workspace = Workspace::new(ALLOW)?;
    let output = workspace
        .wrapper()?
        .env("SHIIN_SOCKET", workspace.0.join("missing.sock"))
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(!workspace.0.join("marker.json").exists());
    Ok(())
}

#[test]
fn a_relative_executable_is_rejected_before_execution() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Workspace::new(ALLOW)?;
    let output = Command::new(env!("CARGO_BIN_EXE_shiin"))
        .args([
            "exec",
            "--session",
            "EXAMPLE-wrapper-session",
            "--",
            "relative-program",
        ])
        .current_dir(&workspace.0)
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(!workspace.0.join(".shiin/audit.jsonl").exists());
    Ok(())
}
