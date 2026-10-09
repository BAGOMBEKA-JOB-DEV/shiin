<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0020: Structured intent binding

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Intent binding](../spec/intent-binding.md), [Glossary](../glossary.md#intent-binding), [Glossary](../glossary.md#task-declaration), [Glossary](../glossary.md#constraint), [Use cases](../overview/use-cases.md)

## Context and problem statement

A developer tells an agent: "Fix the login bug. Do not change the database schema."
The agent may be allowed to run database commands, so the only way to enforce the "do not change the schema" instruction is to check each action against the task's constraints.
But natural-language instructions cannot be enforced directly; they must be translated into machine-checkable form.

## Decision drivers

- Task constraints must be explicit and machine-checkable, not natural language.
- A human must confirm the constraints before they apply.
- Natural-language descriptions are recorded for context but never evaluated.

## Considered options

1. Structured task declarations with a closed set of constraint kinds.
2. Free-text task descriptions parsed by an LLM.
3. No intent binding; rely on standing policy alone.

## Decision outcome

Chosen option: "Structured task declarations with a closed set of constraint kinds", because only machine-checkable constraints can be deterministically enforced.

- A task declaration contains a description, constraints, and an expiry.
- Constraints are from a closed set: `scope.paths`, `forbid.action_types`, `allow.hosts`, `limits`, `expires_at`, and `require_pending_for`.
- Intent binding checks each action against the active task's constraints.
- Natural-language descriptions of intent are recorded but never evaluated.
- Task declarations are experimental and marked as such (planned: v0.2).

### Consequences

- Good, because the constraints are deterministic and auditable.
- Good, because a human confirms the constraints, so a model cannot expand the task scope.
- Bad, because the closed constraint set may not cover every task the operator wants to express.

### Confirmation

- The [intent binding specification](../spec/intent-binding.md) defines the constraint schema.
- Use-case [UC-04](../overview/use-cases.md#uc-04-a-task-scoped-session) demonstrates the design.

## Pros and cons of the options

### Structured declarations with closed constraints

- Good, because constraints are enforceable and deterministic.
- Bad, because the closed set may be too narrow.

### Free-text parsed by an LLM

- Good, because it accepts any description.
- Bad, because it is non-deterministic and attackable by prompt injection.

### No intent binding

- Good, because it is simple.
- Bad, because standing policy cannot distinguish an in-task action from an out-of-task one.

## More information

The glossary defines [intent binding](../glossary.md#intent-binding), [task declaration](../glossary.md#task-declaration), and [constraint](../glossary.md#constraint). See [ADR-0009](0009-deterministic-enforcement.md) for why natural-language intent is never evaluated.
