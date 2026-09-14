//! `cargo xtask docs-check`: validates the documentation set.
//!
//! The rules are defined in `docs/project/docs-style-guide.md`.
//! Each submodule implements one group of checks and records problems in a shared [`Report`].

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "docs-check is a terminal command that reports its findings"
)]

mod adrs;
mod ids;
mod metadata;
mod schemas;
mod summary;
mod terms;

use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use regex::Regex;
use walkdir::WalkDir;

/// A Markdown page under `docs/`.
pub(crate) struct Page {
    /// Path relative to `docs/`, with `/` separators.
    pub(crate) rel: String,
    /// File contents.
    pub(crate) text: String,
    /// Metadata parsed from line 1, if present and well-formed.
    pub(crate) meta: Option<metadata::Meta>,
}

impl Page {
    /// Whether the page's metadata declares the given kind.
    pub(crate) fn is_kind(&self, kind: &str) -> bool {
        self.meta.as_ref().is_some_and(|meta| meta.kind == kind)
    }
}

/// Problems found during a run.
#[derive(Default)]
pub(crate) struct Report {
    errors: Vec<String>,
}

impl Report {
    /// Records a problem at a location such as `spec/decision.md:12`.
    pub(crate) fn error(&mut self, location: impl Display, message: impl Display) {
        self.errors.push(format!("{location}: {message}"));
    }
}

/// Regular expressions shared by the checks, compiled once per run.
pub(crate) struct Patterns {
    pub(crate) metadata: Regex,
    pub(crate) milestone: Regex,
    pub(crate) date: Regex,
    pub(crate) summary_link: Regex,
    pub(crate) requirement_definition: Regex,
    pub(crate) requirement_reference: Regex,
    pub(crate) threat_definition: Regex,
    pub(crate) threat_reference: Regex,
    pub(crate) adr_file: Regex,
    pub(crate) adr_reference: Regex,
    pub(crate) requirement_keyword: Regex,
    pub(crate) former_name: Regex,
    pub(crate) mischaracterization: Regex,
    pub(crate) exclusionary: Regex,
    pub(crate) code_block_word: Regex,
    pub(crate) validate_marker: Regex,
    pub(crate) schema_id: Regex,
}

impl Patterns {
    fn compile() -> Result<Self, regex::Error> {
        Ok(Self {
            metadata: Regex::new(
                r"^<!-- shiin-doc: kind=(\S+) status=(\S+) implementation=(\S+) milestone=(\S+) reviewed=(\S+) -->$",
            )?,
            milestone: Regex::new(r"^(m0|v0\.[1-6]|v1\.0|post-1\.0|n/a)$")?,
            date: Regex::new(r"^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$")?,
            summary_link: Regex::new(r"\]\(([^)]*)\)")?,
            requirement_definition: Regex::new(r"\*\*\[([A-Z]+)-(\d{3})\]\*\*")?,
            requirement_reference: Regex::new(
                r"\b(AT|AR|DEC|EVAL|ID|POL|SYN|INT|RISK|APR|AUD|RCPT|ADP|API|SEC)-\d{3}\b",
            )?,
            threat_definition: Regex::new(r"\*\*\[(ST-[STRIDE]-\d{2})\]\*\*")?,
            threat_reference: Regex::new(r"\bST-[STRIDE]-\d{2}\b")?,
            adr_file: Regex::new(r"^adr/(\d{4})-[a-z0-9-]+\.md$")?,
            adr_reference: Regex::new(r"\bADR-(\d{4})\b")?,
            requirement_keyword: Regex::new(
                r"\b(MUST NOT|MUST|SHALL NOT|SHALL|SHOULD NOT|SHOULD|NOT RECOMMENDED|RECOMMENDED|REQUIRED|MAY|OPTIONAL)\b",
            )?,
            former_name: Regex::new(r"(?i)\blatch")?,
            mischaracterization: Regex::new(
                r"(?i)\bshiin is (an? )?(agent )?(firewall|guardrail|sandbox)",
            )?,
            exclusionary: Regex::new(r"(?i)\b(whitelist|blacklist)")?,
            code_block_word: Regex::new(r"`block(ed|s)?`")?,
            validate_marker: Regex::new(r"^<!-- (validate|validate-fail): (\S+) -->$")?,
            schema_id: Regex::new(r"^urn:shiin:schema:([a-z0-9-]+):v(\d+)$")?,
        })
    }
}

/// Prints command-line usage.
pub(crate) fn print_usage() {
    eprintln!("usage: cargo xtask <command>");
    eprintln!();
    eprintln!("commands:");
    eprintln!("  docs-check    validate docs/ against the documentation style guide");
}

/// Runs every documentation check and reports the result.
pub(crate) fn run() -> ExitCode {
    let docs = workspace_root().join("docs");
    let patterns = match Patterns::compile() {
        Ok(patterns) => patterns,
        Err(error) => {
            eprintln!("error: invalid built-in pattern: {error}");
            return ExitCode::FAILURE;
        }
    };
    let pages = match load_pages(&docs, &patterns) {
        Ok(pages) => pages,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut report = Report::default();
    metadata::check(&pages, &patterns, &mut report);
    summary::check(&pages, &patterns, &mut report);
    terms::check(&pages, &patterns, &mut report);
    ids::check(&pages, &patterns, &mut report);
    adrs::check(&pages, &patterns, &mut report);
    schemas::check(&docs, &pages, &patterns, &mut report);

    if report.errors.is_empty() {
        println!("docs-check: {} pages OK", pages.len());
        ExitCode::SUCCESS
    } else {
        for error in &report.errors {
            eprintln!("error: {error}");
        }
        eprintln!(
            "docs-check: {} problem(s) across {} pages",
            report.errors.len(),
            pages.len()
        );
        ExitCode::FAILURE
    }
}

fn workspace_root() -> PathBuf {
    // `xtask/` lives directly under the workspace root.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn load_pages(docs: &Path, patterns: &Patterns) -> Result<Vec<Page>, String> {
    let mut pages = Vec::new();
    for entry in WalkDir::new(docs).sort_by_file_name() {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let rel = relative(docs, path);
        let text = fs::read_to_string(path).map_err(|error| format!("{rel}: {error}"))?;
        let meta = metadata::parse(&text, patterns);
        pages.push(Page { rel, text, meta });
    }
    Ok(pages)
}

/// Returns `path` relative to `base`, with `/` separators on every platform.
pub(crate) fn relative(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Yields lines outside fenced code blocks, with 1-based line numbers.
pub(crate) fn prose_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut open_fence: Option<String> = None;
    text.lines().enumerate().filter_map(move |(index, line)| {
        let trimmed = line.trim_start();
        if let Some(fence) = &open_fence {
            if trimmed.starts_with(fence.as_str()) && trimmed.trim_end().chars().all(|c| fence.starts_with(c)) {
                open_fence = None;
            }
            return None;
        }
        let marker = trimmed.chars().next().filter(|c| *c == '`' || *c == '~')?;
        let run = trimmed.chars().take_while(|c| *c == marker).count();
        if run >= 3 {
            open_fence = Some(marker.to_string().repeat(run));
            return None;
        }
        Some((index + 1, line))
    })
}

/// Removes inline code spans from a line, keeping the surrounding prose.
pub(crate) fn strip_inline_code(line: &str) -> String {
    let mut prose = String::with_capacity(line.len());
    let mut in_code = false;
    for character in line.chars() {
        if character == '`' {
            in_code = !in_code;
        } else if !in_code {
            prose.push(character);
        }
    }
    prose
}

#[cfg(test)]
mod tests {
    use super::{prose_lines, strip_inline_code};

    #[test]
    fn prose_lines_skip_fenced_code_including_nested_fences() {
        let text = "one\n````markdown\n```json\n{}\n```\n````\ntwo\n~~~\nthree\n~~~\nfour";
        let lines: Vec<(usize, &str)> = prose_lines(text).collect();
        assert_eq!(lines, vec![(1, "one"), (7, "two"), (11, "four")]);
    }

    #[test]
    fn strip_inline_code_keeps_only_prose() {
        assert_eq!(strip_inline_code("use `MUST` here but MAY there"), "use  here but MAY there");
    }
}
