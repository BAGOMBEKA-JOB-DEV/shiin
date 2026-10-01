<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0011: Decision values and combining

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Decision](../spec/decision.md), [Evaluation](../spec/evaluation.md), [ADR-0012](0012-evaluation-stage-order.md)

## Decision outcome

The decision values are `allow`, `deny`, and `pending`.
A `deny` from any stage wins.
When no rule matches, the decision is `deny`.
The default decision can be configured to `pending`, but it can never be `allow`.

## Consequences
- Only a policy rule or a grant can produce `allow`.
- Identity, capability, intent, and risk are gates.
