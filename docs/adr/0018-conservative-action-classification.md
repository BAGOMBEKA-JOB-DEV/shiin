<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0018: Conservative action classification

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Action request](../spec/action-request.md), [Classifier](../glossary.md#classifier)

## Decision outcome

Classifiers only ever make decisions stricter.
Anything they cannot classify becomes an opaque effect.
Opaque effects lead to `pending` by default.

## Consequences
- Unknown commands need approval.
- A classifier never invents an action type.
