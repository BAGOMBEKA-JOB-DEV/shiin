//! Checks that the error-code reference matches the source.
//!
//! A code documented as emitted must appear in the crates, and a code marked
//! reserved must not. Without this, the reference drifts into fiction in both
//! directions: codes that were never implemented, and codes that appear in output
//! but are documented nowhere.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use regex::Regex;
use walkdir::WalkDir;

use super::{Page, Report};

/// The page that owns the error-code registry.
const REFERENCE: &str = "reference/error-codes.md";
/// Where the implementation lives.
const SOURCE_DIRECTORY: &str = "crates";

pub(crate) fn check(root: &Path, pages: &[Page], report: &mut Report) {
    let Some(page) = pages.iter().find(|page| page.rel == REFERENCE) else {
        report.error(REFERENCE, "file is missing");
        return;
    };
    let (Ok(row), Ok(code)) = (
        Regex::new(r"^\|\s*`(SHIIN-[A-Z]+-\d{4})`\s*\|[^|]*\|\s*([a-z]+)\s*\|"),
        Regex::new(r"SHIIN-[A-Z]+-\d{4}"),
    ) else {
        report.error("codes", "invalid built-in pattern");
        return;
    };

    let mut documented: BTreeMap<String, String> = BTreeMap::new();
    for line in page.text.lines() {
        if let Some(captures) = row.captures(line)
            && let (Some(name), Some(status)) = (captures.get(1), captures.get(2))
        {
            documented.insert(name.as_str().to_owned(), status.as_str().to_owned());
        }
    }
    if documented.is_empty() {
        report.error(
            REFERENCE,
            "no error codes found: the registry table changed shape",
        );
        return;
    }

    let mut in_source: BTreeSet<String> = BTreeSet::new();
    for entry in WalkDir::new(root.join(SOURCE_DIRECTORY)).sort_by_file_name() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        if let Ok(text) = fs::read_to_string(path) {
            in_source.extend(code.find_iter(&text).map(|found| found.as_str().to_owned()));
        }
    }

    for (name, status) in &documented {
        match status.as_str() {
            "emitted" if !in_source.contains(name) => report.error(
                REFERENCE,
                format!("`{name}` is documented as emitted, but no crate emits it"),
            ),
            "reserved" if in_source.contains(name) => report.error(
                REFERENCE,
                format!("`{name}` is documented as reserved, but a crate emits it"),
            ),
            "emitted" | "reserved" => {}
            other => report.error(
                REFERENCE,
                format!("`{name}` has an unknown status `{other}`; use emitted or reserved"),
            ),
        }
    }
    for name in &in_source {
        if !documented.contains_key(name) {
            report.error(
                SOURCE_DIRECTORY,
                format!("`{name}` appears in the source but is not documented in {REFERENCE}"),
            );
        }
    }
}
