<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0018: Conservative action classification

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Classifier](docs/../glossary.md#classifier), [Action request](../spec/action-request.md), [Risk signals](../spec/risk-signals.md), [Use cases](../overview/use-cases.md)

## Context and problem statement

When an agent runs a command like `python scripts/cleanup.py`, the adapter cannot know what the script will do.
Classifying it as safe would be a security risk; classifying it as dangerous would block too much.
The project needed a classification strategy that errs on the side of caution.

## Decision drivers

- Unknown actions must not be silently allowed.
- Classification must be deterministic and explainable.
- Operators must be able to see why a classification was made.

## Considered options

1. Conservative classification: unknown effects are marked `opaque`, which escalates to `pending`.
2. Heuristic classification: unknown effects are guessed and allowed if low-risk.
3. Static analysis of all script contents.

## Decision outcome

Chosen option: "Conservative classification", because the agent is untrusted and cannot be allowed to hide unknown effects.

- The classifier derives action types and effects from what a host reports.
- Anything the classifier cannot classify becomes an opaque effect.
- Opaque effects raise the `opaque_command` risk signal and produce `pending`.
- Classifiers only ever make decisions stricter: anything they cannot classify becomes opaque.

### Consequences

- Good, because unknown commands are held for approval, not silently allowed.
- Good, because the classification is deterministic and auditable.
- Bad, because some scripts that are genuinely safe are held for approval, increasing prompt fatigue.

### Confirmation

- The [risk signals specification](../spec/risk-signals.md) defines `opaque_command` and other signals.
- Use-case [UC-02](../overview/use-cases.md#uc-02-keep-secrets-out-of-reach) requires shell commands like `cat .env` to be classified as reads of the same canonical path as file-read tools.

## Pros and cons of the options

### Conservative classification

- Good, because it prevents unknown effects from being allowed.
- Bad, because it increases `pending` for legitimate unknown scripts.

### Heuristic classification

- Good, because it reduces friction.
- Bad, because heuristics can be fooled by prompt injection or obfuscated scripts.

### Static analysis

- Good, because it can determine actual effects.
- Bad, because it is fragile, slow, and cannot analyze all languages.

## More information

The glossary defines [opaque effect](../glossary.md#opaque-effect). The [path canonicalization ADR](0019-path-canonicalization.md) describes how paths are normalized before classification.
