<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0012: Evaluation stage order

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Evaluation](../spec/evaluation.md), [ADR-0010](0010-pure-synchronous-engine.md), [ADR-0011](0011-decision-values-and-combining.md), [Glossary](../glossary.md#stage)

## Context and problem statement

The engine evaluates an action request in stages.
Each stage can only make the result stricter, and the order determines which checks happen first.
The project needed to fix the order so that the same inputs always produce the same stages, and so identity and trust are established before expensive policy evaluation.

## Decision drivers

- Identity must be established before anything else, because trust depends on who is asking.
- Capability is a fast pre-filter before policy evaluation.
- Policy is the core decision and must run after identity and capability.
- Intent binding must run after policy, so a task constraint can only restrict, never relax.
- Risk signals must run last, so they can escalate any earlier result.

## Considered options

1. The fixed order: identity, capability, policy, intent, risk.
2. A configurable order where operators choose which stages run first.
3. Parallel evaluation of all stages.

## Decision outcome

Chosen option: "The fixed order: identity, capability, policy, intent, risk", because it matches the principle that later stages can only restrict earlier results, and it ensures identity and capability are checked before the expensive policy evaluation.

- Stages run in exactly this order: identity, capability, policy, intent, and risk.
- The identity and capability stages are gates: they can `deny`, `pending`, or pass, but never `allow`.
- The policy stage can produce `allow`, `deny`, or `pending`.
- The intent stage checks task constraints and can only make the result stricter.
- The risk stage escalates risk signals and can only make the result stricter.

### Consequences

- Good, because the order is deterministic and easy to reason about.
- Good, because identity and capability short-circuit before policy evaluation, saving time.
- Bad, because a fixed order may not be optimal for all workloads; the policy can model most variation within its rules.

### Confirmation

- [Evaluation](../spec/evaluation.md) specifies the combining rules across stages.
- The five-stages table in [How Shiin works](../concepts/how-shiin-works.md#the-five-stages) documents the order for readers.

## Pros and cons of the options

### Fixed order

- Good, because it is predictable and deterministic.
- Bad, because it is not configurable.

### Configurable order

- Good, because operators can optimize for their workload.
- Bad, because it introduces combinatorial complexity into testing and reasoning.

### Parallel evaluation

- Good, because it can be faster.
- Bad, because it breaks the "later stages can only restrict" invariant and complicates the trace.

## More information

The [decision pipeline](../architecture/decision-pipeline.md) describes what data each stage consumes and produces.
