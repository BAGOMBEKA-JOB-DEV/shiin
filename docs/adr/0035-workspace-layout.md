<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0035: Workspace layout

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Workspace and crates](../engineering/workspace-and-crates.md)

## Decision outcome

Shiin is a single Cargo workspace.
Product crates live in `crates/<name>/`.
`xtask/` is repository tooling and is never published.
`fuzz/` has its own workspace and lockfile.

## Consequences
- Crate names are not reserved on crates.io in advance.
- Empty crates are never created.
