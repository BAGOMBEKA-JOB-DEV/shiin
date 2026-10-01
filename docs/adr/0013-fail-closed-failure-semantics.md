<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0013: Fail-closed failure semantics

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Security model](../security/security-model.md), [Error handling](../engineering/error-handling.md), [ADR-0036](0036-error-handling.md)

## Decision outcome

Any failure, error, timeout, or unreadable policy produces `deny`.
A failure that happens after a decision but before the adapter answers also produces `deny`.

## Consequences
- Errors become decisions in exactly one place.
- A crash in the hook binary still answers `deny`.
