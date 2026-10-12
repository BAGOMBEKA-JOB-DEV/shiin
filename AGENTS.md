# Instructions for AI coding agents

This file is for AI assistants working in this repository.
Humans should read [CONTRIBUTING.md](CONTRIBUTING.md) instead, which these rules supplement rather than replace.

Shiin exists because agents act with their operator's authority.
Work here accordingly: stay inside the task you were given, and leave an honest record.

## What this repository is

Shiin is pre-alpha and has no releases, so nothing here is installable from a package manager.

What exists is the documentation set, the specifications and their conformance corpus, the decision records, a Rust tool (`xtask`) that checks the documentation, and the crates that decide actions today: `shiin-schema`, `shiin-classify`, `shiin-policy`, `shiin-engine`, `shiin-audit`, `shiin-hooks`, and the `shiin` command-line interface.

Do not create crates, modules, or source files for unbuilt milestones unless you were asked to implement that milestone. The [roadmap](docs/project/roadmap.md) says which milestone owns what, and each page's banner says whether its subject exists yet.

## Rules

1. **Rust only.** The product and every tool are Rust. Do not add Python, Node, or shell tooling, and do not suggest installing it. Repository automation belongs in `cargo xtask`.
2. **Read the style guide first.** `docs/project/docs-style-guide.md` governs every page under `docs/`: the line-1 metadata comment, the title and banner, requirement identifiers, terminology, and validated examples.
3. **Use the glossary.** `docs/glossary.md` is the only place terms are defined. Decision values are `allow`, `deny`, and `pending`. A human approves; the outcome is allow. Shiin is an authorization layer, never a firewall, guardrail, or sandbox.
4. **Be honest about what exists.** Never describe an unbuilt feature as working, installable, or tested. Pages carry a status banner for exactly this reason; update the metadata when a feature ships, and never before.
5. **Never invent third-party facts.** Claims about Claude Code, Codex CLI, Cursor, MCP, or any other product need a source link and the date it was verified. If you cannot verify it, leave it out and say so.
6. **Never weaken fail-closed behavior.** Any change that could turn a failure, timeout, or unknown input into `allow` needs an accepted decision record. See [ADR-0013](docs/adr/0013-fail-closed-failure-semantics.md).
7. **Treat security paths with care.** For the paths listed in [security-sensitive changes](docs/engineering/security-sensitive-changes.md), include a threat-model impact note and a test or test vector, and expect human review.
8. **Run the checks before you finish.** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask docs-check`, `rumdl check .`, `typos`, `lychee --config lychee.toml './*.md' docs`, and `mdbook build docs`. Report honestly if a check fails or could not be run.
9. **Keep changes scoped.** Edit the files the task calls for. Do not reformat unrelated files, and do not change shared pages such as the glossary, `SUMMARY.md`, or the style guide as a side effect; propose those changes instead.
10. **Never commit secrets,** and never write a realistic-looking credential into an example. Example values contain `EXAMPLE`, and hosts come from the reserved domains in RFC 2606.
11. **A human signs off.** Disclose AI assistance in the pull request, and leave the Developer Certificate of Origin sign-off to the human who takes responsibility.

## When something is unclear

The approved design lives in the documentation.
If a request conflicts with a decision record or a specification, say so rather than quietly doing both.
If a decision genuinely needs to change, that is what the [ADR](docs/adr/README.md) and [RFC](docs/rfcs/README.md) processes are for.
