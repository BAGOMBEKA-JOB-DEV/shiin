//! Checks that reason codes and action types used anywhere are defined in their registries.

use std::collections::BTreeSet;

use regex::Regex;

use super::{Page, Report, prose_lines};

/// The page that owns the reason-code registry.
const DECISION: &str = "spec/decision.md";
/// The page that owns the action-type registry.
const ACTION_TYPES: &str = "spec/action-types.md";

/// Namespaces an action type can start with, so that unrelated dotted names are ignored.
const NAMESPACES: &[&str] = &[
    "fs.", "shell.", "net.", "mcp.", "db.", "vcs.", "pkg.", "agent.",
];

pub(crate) fn check(pages: &[Page], report: &mut Report) {
    let Ok(table_entry) = Regex::new(r"\|\s*`([a-z]+\.[a-z_*]+)`") else {
        report.error("registries", "invalid built-in pattern");
        return;
    };
    let Ok(quoted) = Regex::new(r"`([a-z]+\.[a-z_*]+)`") else {
        report.error("registries", "invalid built-in pattern");
        return;
    };

    let registry = |rel: &str, keep: &dyn Fn(&str) -> bool| -> BTreeSet<String> {
        pages
            .iter()
            .find(|page| page.rel == rel)
            .map(|page| {
                table_entry
                    .captures_iter(&page.text)
                    .filter_map(|captures| captures.get(1))
                    .map(|found| found.as_str().to_owned())
                    .filter(|name| keep(name))
                    .collect()
            })
            .unwrap_or_default()
    };

    let reason_codes = registry(DECISION, &|name: &str| {
        name.starts_with("deny.") || name.starts_with("pending.")
    });
    let action_types = registry(ACTION_TYPES, &|name: &str| {
        NAMESPACES
            .iter()
            .any(|namespace| name.starts_with(namespace))
    });
    if reason_codes.is_empty() {
        report.error(
            DECISION,
            "no reason codes found: the registry table changed shape",
        );
    }
    if action_types.is_empty() {
        report.error(
            ACTION_TYPES,
            "no action types found: the registry table changed shape",
        );
    }

    for page in pages {
        for (number, line) in prose_lines(&page.text) {
            for captures in quoted.captures_iter(line) {
                let Some(name) = captures.get(1).map(|found| found.as_str()) else {
                    continue;
                };
                let location = format!("{}:{number}", page.rel);
                // A reason code is unambiguous; `allow.*` collides with constraint names.
                if (name.starts_with("deny.") || name.starts_with("pending."))
                    && !reason_codes.contains(name)
                {
                    report.error(
                        &location,
                        format!("reason code `{name}` is not in the registry in {DECISION}"),
                    );
                }
                if NAMESPACES
                    .iter()
                    .any(|namespace| name.starts_with(namespace))
                    && !name.ends_with(".*")
                    && !action_types.contains(name)
                {
                    report.error(
                        &location,
                        format!("action type `{name}` is not in the registry in {ACTION_TYPES}"),
                    );
                }
            }
        }
    }
}

/// Checks the `action_type` of every test vector against the registry.
pub(crate) fn check_vector_action_type(
    rel: &str,
    action_type: &str,
    pages: &[Page],
    report: &mut Report,
) {
    let Ok(table_entry) = Regex::new(r"\|\s*`([a-z]+\.[a-z_]+)`") else {
        return;
    };
    let known: BTreeSet<String> = pages
        .iter()
        .find(|page| page.rel == ACTION_TYPES)
        .map(|page| {
            table_entry
                .captures_iter(&page.text)
                .filter_map(|captures| captures.get(1))
                .map(|found| found.as_str().to_owned())
                .collect()
        })
        .unwrap_or_default();
    if !known.is_empty() && !known.contains(action_type) {
        report.error(
            rel,
            format!("action type `{action_type}` is not in the registry in {ACTION_TYPES}"),
        );
    }
}
