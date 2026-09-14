//! Checks terminology rules and the use of requirement keywords.

use super::{Page, Patterns, Report, prose_lines, strip_inline_code};

/// Pages that define the terminology rules and therefore quote the terms they forbid.
const RULE_PAGES: &[&str] = &["glossary.md", "project/docs-style-guide.md"];

/// Pages allowed to mention the project's former working name.
const FORMER_NAME_PAGES: &[&str] = &["adr/0003-project-name-shiin.md", "overview/landscape.md"];

pub(crate) fn check(pages: &[Page], patterns: &Patterns, report: &mut Report) {
    for page in pages.iter().filter(|page| page.rel != "SUMMARY.md") {
        if RULE_PAGES.contains(&page.rel.as_str()) {
            continue;
        }
        let is_spec = page.is_kind("spec");
        for (number, line) in prose_lines(&page.text) {
            let location = format!("{}:{number}", page.rel);
            let prose = strip_inline_code(line);

            if !FORMER_NAME_PAGES.contains(&page.rel.as_str()) && patterns.former_name.is_match(line) {
                report.error(&location, "the former working name may appear only in ADR-0003 and the landscape page");
            }
            if page.rel != "overview/landscape.md" && patterns.mischaracterization.is_match(&prose) {
                report.error(&location, "Shiin is an authorization layer, not a firewall, guardrail, or sandbox");
            }
            if patterns.exclusionary.is_match(line) {
                report.error(&location, "write `allowlist` or `denylist`");
            }
            if patterns.code_block_word.is_match(line) {
                report.error(&location, "the decision value is `deny`, not `block`");
            }
            if !is_spec && !line.starts_with("<!--") {
                if let Some(keyword) = patterns.requirement_keyword.find(&prose) {
                    report.error(
                        &location,
                        format!("requirement keyword `{}` is only allowed on kind=spec pages", keyword.as_str()),
                    );
                }
            }
        }
    }
}
