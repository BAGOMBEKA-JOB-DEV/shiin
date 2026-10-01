<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0030: Audit durability

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Audit events](../spec/audit-events.md), [ADR-0013](0013-fail-closed-failure-semantics.md)

## Decision outcome

A non-read action MUST NOT proceed until its audit event is durably recorded.
A failure that happens after a decision but before the adapter answers produces `deny`.

## Consequences
- The adapter answers `deny` if the audit write fails.
- Durability is tested with fault injection.
