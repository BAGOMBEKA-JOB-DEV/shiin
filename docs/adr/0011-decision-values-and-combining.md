<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0011: Decision values and combining

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Decision](../spec/decision.md), [Evaluation](../spec/evaluation.md), [ADR-0009](0009-deterministic-enforcement.md), [Glossary](../glossary.md#decision)

## Context and problem statement

When multiple stages of evaluation each produce a result, the engine must combine them into a single decision.
The combining logic must be simple enough that an operator can predict the outcome, and it must ensure that a restrictive stage is never overridden by a permissive one.

## Decision drivers

- A `deny` must always win, from any source.
- `pending` must propagate unless a grant already covers it.
- The default when nothing matches must be the safest option.

## Considered options

1. Three values (`allow`, `deny`, `pending`) with deny-wins and default-deny combining.
2. A five-level combining scheme with `permit`, `permit-pending`, `abstain`, `deny-pending`, `deny`.
3. Numeric scoring where higher values win.

## Decision outcome

Chosen option: "Three values with deny-wins and default-deny", because it is the simplest scheme that satisfies every principle in the [vision](../overview/vision.md).

- The decision values are `allow`, `deny`, and `pending`.
- A `deny` from any stage wins over every other result, including grants.
- A `pending` wins over `allow` unless an existing grant covers the request.
- When no rule matches the action, the default decision is `deny`.
- Only policy rules and grants can produce `allow`; gates (identity, capability, intent, risk) can only `deny`, `pending`, or pass.

### Consequences

- Good, because the three-value scheme is easy to explain and reason about.
- Good, because deny-wins prevents a permissive rule from overriding a restrictive gate.
- Bad, because operators sometimes want a "soft deny" that asks rather than hard-stops; this is expressed as `pending` with a reason code.

### Confirmation

- [Evaluation](../spec/evaluation.md) specifies the exact combining algorithm.
- The [decision specification](../spec/decision.md) defines the reason codes for each value.

## Pros and cons of the options

### Three values, deny-wins, default-deny

- Good, because it is simple and matches operator intuition.
- Bad, because it offers no middle ground between `allow` and `deny` other than `pending`.

### Five-level combining

- Good, because it distinguishes soft and hard denials.
- Bad, because it is harder to explain and more error-prone to implement.

### Numeric scoring

- Good, because it can capture nuanced risk.
- Bad, because it conflicts with the determinism commitment in [ADR-0009](0009-deterministic-enforcement.md).

## More information

The glossary defines each decision value and the [reason code](../glossary.md#reason-code) format. The default-decision behavior is configurable to `pending` but can never be `allow`.
