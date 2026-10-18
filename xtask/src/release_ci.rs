//! Repairs and checks the known generated SBOM output reference.

#![expect(
    clippy::print_stderr,
    reason = "repository commands report their findings"
)]

use std::{fs, path::Path, process::ExitCode};

const INCORRECT: &str = "steps.cargo-cyclonedx.output.paths";
const CORRECT: &str = "steps.cargo-cyclonedx.outputs.paths";
const CACHE_TAG: &str = "swatinem/rust-cache@v2";
const CACHE_COMMIT: &str = "swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6";
const SMOKE: &str = "      - name: Validate packaged executables\n        run: cargo run --release --locked --target ${{ matrix.host }} -p xtask -- release-smoke target/distrib\n";
const ATTEST: &str = "      - name: Attest\n";
const CONCURRENCY: &str = "name: Release\nconcurrency:\n  group: release-${{ github.ref }}\n  cancel-in-progress: ${{ github.event_name == 'pull_request' }}\n";

pub(crate) fn run() -> ExitCode {
    let check = std::env::args().any(|argument| argument == "--check");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows/release.yml");
    let result = fs::read_to_string(&path)
        .map_err(|error| error.to_string())
        .and_then(|text| {
            let fixed = repair(&text)?;
            if check && text != fixed {
                return Err("run cargo xtask release-ci-fix after dist generate".to_owned());
            }
            if text != fixed {
                fs::write(path, fixed).map_err(|error| error.to_string())?;
            }
            Ok(())
        });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("release-ci-fix: {error}");
            ExitCode::FAILURE
        }
    }
}

fn repair(text: &str) -> Result<String, String> {
    let mut fixed = text.to_owned();
    for (original, replacement) in [(INCORRECT, CORRECT), (CACHE_TAG, CACHE_COMMIT)] {
        match (
            fixed.matches(original).count(),
            fixed.matches(replacement).count(),
        ) {
            (1, 0) => fixed = fixed.replacen(original, replacement, 1),
            (0, 1) => {}
            _ => {
                return Err(
                    "generated release template changed; review its SBOM reference and cache pin"
                        .to_owned(),
                );
            }
        }
    }
    if !fixed.contains(CONCURRENCY) {
        if fixed.matches("name: Release\n").count() != 1 || fixed.contains("concurrency:") {
            return Err(
                "generated release concurrency changed; review before publication".to_owned(),
            );
        }
        fixed = fixed.replacen("name: Release\n", CONCURRENCY, 1);
    }
    if !fixed.contains(SMOKE) {
        if fixed.matches(ATTEST).count() != 1 {
            return Err("generated archive validation position changed".to_owned());
        }
        fixed = fixed.replacen(ATTEST, &format!("{SMOKE}{ATTEST}"), 1);
    }
    Ok(fixed)
}

#[cfg(test)]
mod tests {
    use super::{ATTEST, CACHE_COMMIT, CACHE_TAG, CORRECT, INCORRECT, SMOKE, repair};

    #[test]
    fn fixes_only_one_expected_reference_and_is_idempotent() {
        let input =
            format!("name: Release\nbefore ${{{{ {INCORRECT} }}}} {CACHE_TAG} after\n{ATTEST}");
        let fixed = repair(&input).expect("recognized generated reference");
        assert!(!fixed.contains(INCORRECT));
        assert!(!fixed.contains(CACHE_TAG));
        assert!(fixed.contains(CORRECT));
        assert!(fixed.contains(CACHE_COMMIT));
        assert!(fixed.contains(&format!("{SMOKE}{ATTEST}")));
        assert_eq!(repair(&fixed).expect("already repaired"), fixed);
        assert!(repair("unknown template").is_err());
        assert!(repair(&format!("{INCORRECT} {INCORRECT}")).is_err());
        assert!(repair(&format!("{INCORRECT} {CORRECT}")).is_err());
    }
}
