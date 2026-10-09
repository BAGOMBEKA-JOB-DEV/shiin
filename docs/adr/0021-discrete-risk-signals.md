<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0021: Discrete risk signals

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Risk signals](../spec/risk-signals.md), [Glossary](../glossary.md#risk-signal), [How Shiin works](../concepts/how-shiin-works.md), [ADR-0009](0009-deterministic-enforcement.md)

## Context and problem statement

Some actions are riskier than others: a bulk deletion, reading a secret path, or running an opaque script.
The engine must escalate these to `pending` for human review.
But a numeric risk score is non-deterministic and can be gamed.
The project needed a risk model that is deterministic and explainable.

## Decision drivers

- Risk assessment must be deterministic (see [ADR-0009](0009-deterministic-enforcement.md)).
- Operators must be able to understand why a signal fired.
- Risk must never be used to allow an action that would otherwise be denied.

## Considered options

1. Discrete, deterministic risk signals (named properties) that escalate to `pending`.
2. A numeric risk score where higher scores escalate.
3. A machine-learning model that classifies risk.

## Decision outcome

Chosen option: "Discrete, deterministic risk signals", because a named property is reproducible and explainable.

- Risk signals are discrete, deterministic properties of an action request, such as `bulk_delete` or `secret_path`.
- A signal escalates the decision to `pending` unless a matching grant already covers it.
- Signals never allow; they can only restrict or escalate.
- There are no numeric risk scores.

### Consequences

- Good, because each signal is reproducible and auditable.
- Good, because operators can see exactly which signal fired and why.
- Bad, because discrete signals cannot capture subtle gradations of risk.

### Confirmation

- The [risk signals specification](../spec/risk-signals.md) defines the full set of signals, their triggering conditions, and their reason codes.
- Use-cases [UC-01](../overview/use-cases.md#uc-01-one-policy-across-several-agents), [UC-02](../overview/use-cases.md#uc-02-keep-secrets-out-of-reach), and [UC-03](../overview/use-cases.md#uc-03-guard-against-bulk-deletion) define the core signals.

## Pros and cons of the options

### Discrete risk signals

- Good, because they are deterministic and explainable.
- Bad, because they cannot express gradations.

### Numeric risk scoring

- Good, because it can capture nuance.
- Bad, because it conflicts with determinism and can be gamed.

### Machine-learning risk model

- Good, because it can detect subtle patterns.
- Bad, because it is non-deterministic, unexplainable, and trainable by attackers.

## More information

Each risk signal produces a reason code starting with `pending.` (for example, `pending.opaque` in [How Shiin works](../concepts/how-shiin-works.md)). The full catalog is in the [risk signals specification](../spec/risk-signals.md).
