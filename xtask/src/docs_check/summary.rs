//! Checks that `SUMMARY.md` and the pages on disk match in both directions.

use std::collections::BTreeSet;

use super::{Page, Patterns, Report};

pub(crate) fn check(pages: &[Page], patterns: &Patterns, report: &mut Report) {
    let Some(summary) = pages.iter().find(|page| page.rel == "SUMMARY.md") else {
        report.error("SUMMARY.md", "file is missing");
        return;
    };
    let existing: BTreeSet<&str> = pages.iter().map(|page| page.rel.as_str()).collect();
    let mut listed = BTreeSet::new();

    for (number, line) in summary.text.lines().enumerate() {
        for captures in patterns.summary_link.captures_iter(line) {
            let Some(target) = captures.get(1).map(|m| m.as_str()) else {
                continue;
            };
            // An empty target is an mdBook draft chapter, used as a section title.
            if target.is_empty() {
                continue;
            }
            let path = target.split('#').next().unwrap_or(target);
            let location = format!("SUMMARY.md:{}", number + 1);
            if !existing.contains(path) {
                report.error(&location, format!("entry `{path}` does not exist"));
            }
            if !listed.insert(path.to_owned()) {
                report.error(&location, format!("entry `{path}` is listed more than once"));
            }
        }
    }

    for page in pages.iter().filter(|page| page.rel != "SUMMARY.md") {
        if !listed.contains(&page.rel) {
            report.error(&page.rel, "page is not listed in SUMMARY.md");
        }
    }
}
