//! Checks architecture decision records: numbering, titles, the index, and references.

use std::collections::BTreeMap;

use super::{Page, Patterns, Report, prose_lines};

const INDEX: &str = "adr/README.md";

pub(crate) fn check(pages: &[Page], patterns: &Patterns, report: &mut Report) {
    let mut records: BTreeMap<String, &Page> = BTreeMap::new();
    for page in pages {
        let Some(number) = patterns
            .adr_file
            .captures(&page.rel)
            .and_then(|captures| captures.get(1))
            .map(|m| m.as_str().to_owned())
        else {
            continue;
        };
        if let Some(previous) = records.insert(number.clone(), page) {
            report.error(&page.rel, format!("ADR number {number} is already used by {}", previous.rel));
        }
        let expected_title = format!("# ADR-{number}: ");
        if !page.text.lines().nth(2).is_some_and(|line| line.starts_with(&expected_title)) {
            report.error(format!("{}:3", page.rel), format!("title must start with `{expected_title}`"));
        }
    }

    match pages.iter().find(|page| page.rel == INDEX) {
        None => report.error(INDEX, "file is missing"),
        Some(index) => {
            for page in records.values() {
                let file_name = page.rel.trim_start_matches("adr/");
                if !index.text.contains(&format!("({file_name})")) {
                    report.error(INDEX, format!("index does not link to `{file_name}`"));
                }
            }
        }
    }

    for page in pages {
        for (number, line) in prose_lines(&page.text) {
            for captures in patterns.adr_reference.captures_iter(line) {
                if let Some(referenced) = captures.get(1) {
                    if !records.contains_key(referenced.as_str()) {
                        report.error(
                            format!("{}:{number}", page.rel),
                            format!("ADR-{} does not exist", referenced.as_str()),
                        );
                    }
                }
            }
        }
    }
}
