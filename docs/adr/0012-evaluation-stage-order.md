<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0012: Evaluation stage order

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Evaluation](../spec/evaluation.md), [ADR-0011](0011-decision-values-and-combining.md)

## Decision outcome

The engine runs exactly five stages in this fixed order: identity, capability, policy, intent, and risk.
Later stages can only make the result stricter.

## Consequences
- The order is stable and tested.
- A trace records each stage's result.
