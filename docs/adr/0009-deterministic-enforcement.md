<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0009: Deterministic enforcement

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Evaluation](../spec/evaluation.md), [ADR-0010](0010-pure-synchronous-engine.md)

## Decision outcome

The same action request and the same policy snapshot always produce the same decision.
No language-model or machine-learning output is an input to any decision.

## Consequences
- Replay reproduces every decision in the conformance corpus.
- The engine is pure and synchronous.