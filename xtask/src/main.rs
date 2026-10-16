//! Repository automation for the Shiin workspace.
//!
//! Run `cargo xtask <command>`. Commands:
//!
//! - `docs-check`: validate `docs/` against `docs/project/docs-style-guide.md`.
//! - `intent-evaluation`: measure task constraints against the labelled corpus.
//! - `dependency-check`: refuse network stacks and async runtimes in engine crates.
//! - `fuzz-smoke`: seed and run bounded parser fuzzing on the pinned nightly.
//! - `native-keyring-check`: validate native key-store persistence and rotation in CI.
//! - `memory`: check bounded component heap use with `--features memory-profile`.

mod conformance;
mod dependency_check;
mod docs_check;
mod fuzz_smoke;
mod intent_evaluation;
#[cfg(feature = "memory-profile")]
mod memory;
mod native_keyring;
mod performance;
mod performance_scaling;

use std::process::ExitCode;

fn main() -> ExitCode {
    #[cfg(feature = "memory-profile")]
    if std::env::args().nth(1).as_deref() == Some("memory") {
        return memory::run();
    }
    if std::env::args().nth(1).as_deref() == Some("native-keyring-check") {
        return native_keyring::run();
    }
    if std::env::args().nth(1).as_deref() == Some("conformance") {
        return conformance::run();
    }
    if std::env::args().nth(1).as_deref() == Some("performance-scaling") {
        return performance_scaling::run();
    }
    if std::env::args().nth(1).as_deref() == Some("fuzz-smoke") {
        return fuzz_smoke::run();
    }
    if std::env::args().nth(1).as_deref() == Some("dependency-check") {
        return dependency_check::run();
    }
    if std::env::args().nth(1).as_deref() == Some("intent-evaluation") {
        return intent_evaluation::run();
    }
    if std::env::args().nth(1).as_deref() == Some("performance") {
        return performance::run();
    }
    if std::env::args().nth(1).as_deref() == Some("docs-check") {
        return docs_check::run();
    }
    docs_check::print_usage();
    ExitCode::FAILURE
}
