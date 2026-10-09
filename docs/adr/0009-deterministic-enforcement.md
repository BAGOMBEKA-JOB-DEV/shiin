<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0009: Deterministic enforcement

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Vision](../overview/vision.md#principles), [Evaluation](../spec/evaluation.md), [ADR-0010](0010-pure-synchronous-engine.md), [ADR-0011](0011-decision-values-and-combining.md)

## Context and problem statement

An authorization layer that sometimes allows and sometimes denies the same action depending on hidden state is worse than useless: it trains the operator not to trust it.
The engine needed to guarantee that identical inputs produce identical outputs, with no dependence on timing, network state, or language-model output.

## Decision drivers

- Replay must reproduce decisions exactly.
- Security auditors must be able to reason about the engine's behavior from a specification.
- No input to any decision may come from a language model or machine-learning model.

## Considered options

1. Deterministic evaluation: the same action request and policy snapshot always produce the same decision.
2. Probabilistic risk scoring: a numeric risk score determines whether an action is allowed.
3. Model-in-the-loop: a language model judges each action.

## Decision outcome

Chosen option: "Deterministic evaluation", because it makes the engine's behavior reproducible, auditable, and testable without statistical uncertainty.

- The engine is a pure function of the action request and the policy snapshot.
- No language-model output, machine-learning model, or timing-dependent input influences any stage.
- Risk signals are discrete properties (such as `bulk_delete`) that escalate a decision to `pending`, never numeric scores.
- Determinism is a design commitment referenced from the [vision](../overview/vision.md#principles).

### Consequences

- Good, because replay and verification are meaningful.
- Good, because attackers cannot influence decisions through timing or state manipulation.
- Bad, because the engine cannot adapt to context that a model might detect, such as subtle prompt-injection signals.

### Confirmation

- [ADR-0021](0021-discrete-risk-signals.md) requires risk signals to be deterministic properties.
- [ADR-0010](0010-pure-synchronous-engine.md) makes the engine pure, which is a prerequisite for determinism.

## Pros and cons of the options

### Deterministic evaluation

- Good, because decisions are reproducible and auditable.
- Bad, because it cannot adapt to statistical signals.

### Probabilistic risk scoring

- Good, because numeric scores can capture nuanced risk.
- Bad, because non-determinism makes replay impossible and models can be manipulated.

### Model-in-the-loop

- Good, because models can understand intent.
- Bad, because no language model is reliably safe, and the output is non-deterministic.

## More information

ADR-0009 is the foundation of the [evaluation specification](../spec/evaluation.md). The [risk signals specification](../spec/risk-signals.md) defines which signals exist and how they escalate decisions.
