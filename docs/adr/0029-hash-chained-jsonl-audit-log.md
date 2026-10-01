<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0029: Hash-chained JSON Lines audit log

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Audit events](../spec/audit-events.md), [Audit log](../glossary.md#audit-log)

## Decision outcome

The audit log is an append-only JSON Lines file.
Each event includes the hash of the event before it, forming a hash chain.
The audit log is tamper-evident, not tamper-proof.

## Consequences
- Removing or altering an event is detectable.
- Verification checks the hash chain.
