<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0036: Error handling

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Error handling](../engineering/error-handling.md), [ADR-0013](0013-fail-closed-failure-semantics.md)

## Decision outcome

Every unhappy outcome belongs to one of four categories: decisions, operational errors, user diagnostics, or bugs.
Decisions are not errors.
Operational errors are per-crate thiserror enums marked `#[non_exhaustive]`.
Errors become decisions in exactly one place, which produces `deny`.

## Consequences
- Libraries do not use anyhow or Box<dyn Error>.
- Every error has a stable code.
