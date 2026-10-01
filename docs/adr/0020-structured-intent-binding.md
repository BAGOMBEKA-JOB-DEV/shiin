<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0020: Structured intent binding

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Intent binding](../spec/intent-binding.md), [Tasks and intent](../concepts/tasks-and-intent.md)

## Decision outcome

Task constraints are explicit and machine-checkable, and a human confirms them.
Natural-language descriptions of intent are recorded but never evaluated.

## Consequences
- The constraint set is closed.
- A model never decides what the task allows.
