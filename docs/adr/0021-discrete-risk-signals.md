<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0021: Discrete risk signals

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Risk signals](../spec/risk-signals.md)

## Decision outcome

Risk signals are discrete, deterministic properties of an action request.
Shiin does not use numeric risk scores.
Each signal has a configured decision: `deny`, `pending`, or `pass`.

## Consequences
- Signals are easy to test and explain.
- A signal that is not configured produces `pass`.
