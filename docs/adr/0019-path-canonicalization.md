<!-- shiin-doc: kind=adr status=proposed implementation=n/a milestone=v0.1 reviewed=2026-10-01 -->

# ADR-0019: Path canonicalization

> [!NOTE]
> **Architecture decision: proposed.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Action request](../spec/action-request.md), [ADR-0018](0018-conservative-action-classification.md)

## Decision outcome

Paths are canonicalized before evaluation.
Canonicalization resolves symlinks, relative paths, and `..` segments.
The result is recorded with confidence `exact`.

## Consequences
- A rule on a canonical path covers all ways to name it.
- A path that cannot be canonicalized becomes `inferred` or `opaque`.
