//! Informational commands must never start the daemon or create runtime state.

use std::process::Command;

#[test]
fn informational_and_invalid_arguments_do_not_start_a_service() {
    let directory = std::env::temp_dir().join(format!("shiin-EXAMPLE-help-{}", std::process::id()));
    let endpoint = directory.join("shiind.sock");
    for argument in ["--version", "--help", "--EXAMPLE-invalid"] {
        let output = Command::new(env!("CARGO_BIN_EXE_shiind"))
            .arg(argument)
            .env("SHIIN_SOCKET", &endpoint)
            .output()
            .expect("run daemon information command");
        assert_eq!(output.status.success(), argument != "--EXAMPLE-invalid");
        if argument == "--version" {
            assert!(String::from_utf8_lossy(&output.stdout).contains(env!("CARGO_PKG_VERSION")));
        }
        assert!(
            !directory.exists(),
            "information commands created daemon state"
        );
    }
}
