//! Checks requirement IDs and threat IDs: unique definitions, correct owners, and resolvable references.

use std::collections::BTreeMap;

use super::{Page, Patterns, Report, prose_lines};

/// Each requirement ID prefix and the page that owns it.
const PREFIX_OWNERS: &[(&str, &str)] = &[
    ("AT", "spec/action-types.md"),
    ("AR", "spec/action-request.md"),
    ("DEC", "spec/decision.md"),
    ("EVAL", "spec/evaluation.md"),
    ("ID", "spec/identity.md"),
    ("POL", "spec/policy-model.md"),
    ("SYN", "spec/policy-syntax.md"),
    ("INT", "spec/intent-binding.md"),
    ("RISK", "spec/risk-signals.md"),
    ("APR", "spec/approval-protocol.md"),
    ("AUD", "spec/audit-events.md"),
    ("RCPT", "spec/receipts.md"),
    ("ADP", "spec/adapter-contract.md"),
    ("API", "spec/local-api.md"),
    ("SEC", "security/security-model.md"),
];

const THREAT_MODEL: &str = "security/threat-model.md";

pub(crate) fn check(pages: &[Page], patterns: &Patterns, report: &mut Report) {
    let requirements = collect_requirement_definitions(pages, patterns, report);
    let threats = collect_threat_definitions(pages, patterns, report);

    for page in pages {
        for (number, line) in prose_lines(&page.text) {
            let location = format!("{}:{number}", page.rel);
            for found in patterns.requirement_reference.find_iter(line) {
                if !requirements.contains_key(found.as_str()) {
                    report.error(&location, format!("requirement `{}` is not defined", found.as_str()));
                }
            }
            for found in patterns.threat_reference.find_iter(line) {
                if !threats.contains_key(found.as_str()) {
                    report.error(&location, format!("threat `{}` is not defined", found.as_str()));
                }
            }
        }
    }
}

fn collect_requirement_definitions(
    pages: &[Page],
    patterns: &Patterns,
    report: &mut Report,
) -> BTreeMap<String, String> {
    let mut defined = BTreeMap::new();
    for page in pages {
        for (number, line) in prose_lines(&page.text) {
            let location = format!("{}:{number}", page.rel);
            for captures in patterns.requirement_definition.captures_iter(line) {
                let (Some(prefix), Some(digits)) = (captures.get(1), captures.get(2)) else {
                    continue;
                };
                let id = format!("{}-{}", prefix.as_str(), digits.as_str());
                match PREFIX_OWNERS.iter().find(|(known, _)| *known == prefix.as_str()) {
                    None => report.error(&location, format!("unknown requirement prefix in `{id}`")),
                    Some((_, owner)) if *owner != page.rel => {
                        report.error(&location, format!("`{id}` must be defined in {owner}"));
                    }
                    Some(_) => {}
                }
                if !page.is_kind("spec") {
                    report.error(&location, format!("`{id}` is defined on a page that is not kind=spec"));
                }
                if let Some(previous) = defined.insert(id.clone(), location.clone()) {
                    report.error(&location, format!("`{id}` is already defined at {previous}"));
                }
            }
        }
    }
    defined
}

fn collect_threat_definitions(
    pages: &[Page],
    patterns: &Patterns,
    report: &mut Report,
) -> BTreeMap<String, String> {
    let mut defined = BTreeMap::new();
    for page in pages {
        for (number, line) in prose_lines(&page.text) {
            let location = format!("{}:{number}", page.rel);
            for captures in patterns.threat_definition.captures_iter(line) {
                let Some(id) = captures.get(1).map(|m| m.as_str().to_owned()) else {
                    continue;
                };
                if page.rel != THREAT_MODEL {
                    report.error(&location, format!("threat `{id}` must be defined in {THREAT_MODEL}"));
                }
                if let Some(previous) = defined.insert(id.clone(), location.clone()) {
                    report.error(&location, format!("threat `{id}` is already defined at {previous}"));
                }
            }
        }
    }
    defined
}
