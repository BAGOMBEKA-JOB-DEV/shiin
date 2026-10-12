//! Repository automation for the Shiin workspace.
//!
//! Run `cargo xtask <command>`. Commands:
//!
//! - `docs-check`: validate `docs/` against `docs/project/docs-style-guide.md`.

mod docs_check;

use std::process::ExitCode;

fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() == Some("docs-check") {
        return docs_check::run();
    }
    docs_check::print_usage();
    ExitCode::FAILURE
}
