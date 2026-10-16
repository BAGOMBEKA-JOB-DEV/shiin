//! Opt-in native key-store lifecycle checks for ephemeral hosted runners.

#![expect(
    clippy::print_stderr,
    reason = "repository checks report their failures"
)]

use std::process::{Command, ExitCode};

pub(crate) fn run() -> ExitCode {
    if std::env::var("CI").as_deref() != Ok("true") {
        eprintln!("native-keyring-check requires an ephemeral CI runner");
        return ExitCode::FAILURE;
    }
    match check() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("native key-store validation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let _daemon = linux_store()?;
    let status = Command::new("cargo")
        .args([
            "test",
            "--locked",
            "-p",
            "shiin-audit",
            "--features",
            "keyring",
            "--lib",
            "keys::tests::native_key_store_lifecycle",
            "--",
            "--ignored",
            "--exact",
        ])
        .env("SHIIN_TEST_NATIVE_KEYRING", "1")
        .status()
        .map_err(|error| format!("could not run native key-store test: {error}"))?;
    if !status.success() {
        return Err("native backend lifecycle test failed".to_owned());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
struct Daemon(std::process::Child);

#[cfg(target_os = "linux")]
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(target_os = "linux")]
fn linux_store() -> Result<Daemon, String> {
    use std::{
        io::Write as _,
        process::Stdio,
        time::{Duration, Instant},
    };

    if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
        return Err("run this command under dbus-run-session".to_owned());
    }
    let mut daemon = Daemon(
        Command::new("gnome-keyring-daemon")
            .args(["--unlock", "--foreground", "--components=secrets"])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|error| format!("could not start test Secret Service: {error}"))?,
    );
    daemon
        .0
        .stdin
        .take()
        .ok_or("test daemon stdin unavailable")?
        .write_all(b"EXAMPLE ephemeral CI keyring password")
        .map_err(|error| format!("could not unlock test Secret Service: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if daemon
            .0
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Err("test Secret Service exited before becoming available".to_owned());
        }
        let response = Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.DBus",
                "--type=method_call",
                "--print-reply",
                "--reply-timeout=1000",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus.NameHasOwner",
                "string:org.freedesktop.secrets",
            ])
            .output()
            .map_err(|error| format!("could not query test Secret Service: {error}"))?;
        if response.status.success()
            && String::from_utf8_lossy(&response.stdout).contains("boolean true")
        {
            return Ok(daemon);
        }
        if Instant::now() >= deadline {
            return Err("test Secret Service did not start within ten seconds".to_owned());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
