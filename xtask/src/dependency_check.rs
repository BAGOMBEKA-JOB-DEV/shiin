//! Enforces the engine crates' prohibition on network stacks and async runtimes.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "repository checks report their findings"
)]

use std::{
    collections::BTreeSet,
    process::{Command, ExitCode},
};

const ENGINE_CRATES: &[&str] = &[
    "shiin-schema",
    "shiin-classify",
    "shiin-policy",
    "shiin-engine",
];
const FORBIDDEN: &[&str] = &[
    "tokio",
    "async-std",
    "smol",
    "mio",
    "socket2",
    "reqwest",
    "ureq",
    "surf",
    "hyper",
    "hyper-util",
    "axum",
    "warp",
    "actix-web",
    "rmcp",
    "quinn",
    "quinn-proto",
    "rustls",
    "native-tls",
    "openssl",
    "ssh2",
    "russh",
    "tungstenite",
    "tokio-tungstenite",
];

pub(crate) fn run() -> ExitCode {
    match check() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dependency-check: {error}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for package in ENGINE_CRATES {
        let output = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .current_dir(&root)
            .args([
                "tree",
                "--locked",
                "--offline",
                "--edges",
                "normal",
                "--prefix",
                "none",
                "--package",
                package,
            ])
            .output()
            .map_err(|_| "could not inspect Cargo's dependency graph")?;
        if !output.status.success() {
            return Err(format!("Cargo could not inspect {package}'s dependencies"));
        }
        let tree =
            String::from_utf8(output.stdout).map_err(|_| "invalid Cargo dependency output")?;
        let violations = violations(&tree);
        if !violations.is_empty() {
            return Err(format!(
                "{package} depends on prohibited crates: {}",
                violations.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
        println!("dependency-check: {package}: no prohibited network or runtime dependency");
    }
    Ok(())
}

fn violations(tree: &str) -> BTreeSet<String> {
    tree.lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| FORBIDDEN.contains(name))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::violations;

    #[test]
    fn transitive_network_and_runtime_crates_are_refused() {
        let tree = "shiin-engine v0.0.0\nshiin-policy v0.0.0\nreqwest v0.13.5\nhyper v1.12.0\ntokio v1.53.0\nhyper v1.12.0 (*)\n";
        assert_eq!(
            violations(tree).into_iter().collect::<Vec<_>>(),
            ["hyper", "reqwest", "tokio"]
        );
        assert!(
            violations(
                "shiin-engine v0.0.0\nserde v1.0.229\ncedar-policy v4.13.0\nsmol_str v0.3.6\n"
            )
            .is_empty()
        );
    }
}
