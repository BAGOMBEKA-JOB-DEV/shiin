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
    Ok(fixed)
}

#[cfg(test)]
mod tests {
    use super::{CACHE_COMMIT, CACHE_TAG, CORRECT, INCORRECT, repair};

    #[test]
    fn fixes_only_one_expected_reference_and_is_idempotent() {
        let input = format!("before ${{{{ {INCORRECT} }}}} {CACHE_TAG} after");
        let fixed = repair(&input).expect("recognized generated reference");
        assert_eq!(
            fixed,
            format!("before ${{{{ {CORRECT} }}}} {CACHE_COMMIT} after")
        );
        assert_eq!(repair(&fixed).expect("already repaired"), fixed);
        assert!(repair("unknown template").is_err());
        assert!(repair(&format!("{INCORRECT} {INCORRECT}")).is_err());
        assert!(repair(&format!("{INCORRECT} {CORRECT}")).is_err());
    }
}
