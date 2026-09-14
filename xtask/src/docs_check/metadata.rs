//! Checks the metadata comment, title, and status banner at the top of every page.

use super::{Page, Patterns, Report};

const KINDS: &[&str] = &[
    "index",
    "explanation",
    "spec",
    "reference",
    "how-to",
    "tutorial",
    "adr",
    "rfc",
    "policy",
];
const STATUSES: &[&str] = &[
    "draft",
    "proposed",
    "accepted",
    "deferred",
    "superseded",
    "deprecated",
];
const IMPLEMENTATIONS: &[&str] = &["none", "partial", "complete", "n/a"];

/// Metadata declared on line 1 of a page.
pub(crate) struct Meta {
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) implementation: String,
    pub(crate) milestone: String,
    pub(crate) reviewed: String,
}

/// Parses the metadata comment on the first line, if it has the expected shape.
pub(crate) fn parse(text: &str, patterns: &Patterns) -> Option<Meta> {
    let captures = patterns.metadata.captures(text.lines().next()?)?;
    let field = |index| captures.get(index).map_or_else(String::new, |m| m.as_str().to_owned());
    Some(Meta {
        kind: field(1),
        status: field(2),
        implementation: field(3),
        milestone: field(4),
        reviewed: field(5),
    })
}

pub(crate) fn check(pages: &[Page], patterns: &Patterns, report: &mut Report) {
    for page in pages.iter().filter(|page| page.rel != "SUMMARY.md") {
        let Some(meta) = &page.meta else {
            report.error(
                format!("{}:1", page.rel),
                "line 1 must be a `<!-- shiin-doc: ... -->` metadata comment",
            );
            continue;
        };
        check_fields(page, meta, patterns, report);
        check_location(page, meta, patterns, report);
        check_title_and_banner(page, meta, report);
    }
}

fn check_fields(page: &Page, meta: &Meta, patterns: &Patterns, report: &mut Report) {
    let location = format!("{}:1", page.rel);
    if !KINDS.contains(&meta.kind.as_str()) {
        report.error(&location, format!("unknown kind `{}`", meta.kind));
    }
    if !STATUSES.contains(&meta.status.as_str()) {
        report.error(&location, format!("unknown status `{}`", meta.status));
    }
    if !IMPLEMENTATIONS.contains(&meta.implementation.as_str()) {
        report.error(&location, format!("unknown implementation `{}`", meta.implementation));
    }
    if !patterns.milestone.is_match(&meta.milestone) {
        report.error(&location, format!("unknown milestone `{}`", meta.milestone));
    }
    if !patterns.date.is_match(&meta.reviewed) {
        report.error(&location, format!("`reviewed` must be YYYY-MM-DD, found `{}`", meta.reviewed));
    }
    if meta.implementation == "n/a" && matches!(meta.kind.as_str(), "spec" | "tutorial" | "how-to") {
        report.error(&location, format!("kind `{}` must not use implementation=n/a", meta.kind));
    }
}

fn check_location(page: &Page, meta: &Meta, patterns: &Patterns, report: &mut Report) {
    let is_record = patterns.adr_file.is_match(&page.rel) || page.rel == "adr/template.md";
    if is_record && meta.kind != "adr" {
        report.error(format!("{}:1", page.rel), "architecture decision records must use kind=adr");
    }
    if meta.kind == "adr" && !is_record {
        report.error(
            format!("{}:1", page.rel),
            "kind=adr is only for `adr/NNNN-*.md` and `adr/template.md`",
        );
    }
}

fn check_title_and_banner(page: &Page, meta: &Meta, report: &mut Report) {
    let lines: Vec<&str> = page.text.lines().collect();
    let line = |number: usize| lines.get(number - 1).copied().unwrap_or_default();

    if !line(2).is_empty() {
        report.error(format!("{}:2", page.rel), "line 2 must be blank");
    }
    if !line(3).starts_with("# ") {
        report.error(format!("{}:3", page.rel), "line 3 must be the level-1 title (`# Title`)");
    }
    if !line(4).is_empty() {
        report.error(format!("{}:4", page.rel), "line 4 must be blank");
    }

    let (admonition, label) = expected_banner(meta);
    let marker = format!("> [!{admonition}]");
    if line(5) != marker {
        report.error(format!("{}:5", page.rel), format!("banner must start with `{marker}`"));
    }
    let label_line = format!("> **{label}.**");
    if !line(6).starts_with(&label_line) {
        report.error(format!("{}:6", page.rel), format!("banner label must be `{label_line}`"));
    }
}

/// The admonition type and label a page's banner must use, derived from its metadata.
fn expected_banner(meta: &Meta) -> (&'static str, String) {
    let status = &meta.status;
    match meta.kind.as_str() {
        "index" => ("NOTE", "Project status: pre-alpha".to_owned()),
        "spec" => ("NOTE", format!("Specification: {status}")),
        "explanation" => ("NOTE", format!("Design document: {status}")),
        "adr" => ("NOTE", format!("Architecture decision: {status}")),
        "rfc" => ("NOTE", format!("RFC: {status}")),
        "policy" => ("NOTE", format!("Project policy: {status}")),
        "tutorial" | "how-to" | "reference" if meta.implementation == "none" => {
            ("WARNING", "Design intent: not implemented".to_owned())
        }
        "tutorial" | "how-to" => ("NOTE", format!("Guide: {status}")),
        _ => ("NOTE", format!("Reference: {status}")),
    }
}
