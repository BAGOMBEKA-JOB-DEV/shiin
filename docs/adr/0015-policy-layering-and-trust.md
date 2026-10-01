<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0015: Policy layering and trust

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Policy model](../spec/policy-model.md)

## Decision outcome

Policy is drawn from layers, in order: built-in, user, and workspace.
A workspace layer can restrict a user layer only; it can never make a deny into an allow unless the operator trusts it by hash.
An organization layer is reserved for the future.

## Consequences
- A workspace policy cannot escalate permissions.
- The operator controls trust by hash.
