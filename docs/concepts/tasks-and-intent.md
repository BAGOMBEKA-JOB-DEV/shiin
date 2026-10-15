<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.2 reviewed=2026-10-15 -->

# Tasks and intent

> [!NOTE]
> **Guide: draft.**
> The source build implements the current behavior below; remaining design work is marked explicitly.

A [task declaration](../glossary.md#task-declaration) is a structured, human-confirmed statement of what an agent is working on: a description, [constraints](../glossary.md#constraint), and an expiry.

## Intent binding

[Intent binding](../glossary.md#intent-binding) checks an [action](../glossary.md#action) against the constraints of the active task.
Natural-language descriptions of intent are recorded but never evaluated.

## Constraints

The constraint set is closed:

| Constraint | Meaning |
|---|---|
| `scope.paths` | Paths the agent may touch. |
| `forbid.action_types` | Action types the agent may not perform. |
| `allow.hosts` | Hosts the agent may contact. |
| `limits` | Counters for actions such as deletions. |
| `expires_at` | When the task ends. |
| `require_pending_for` | Action types that always need approval. |

## Expiry

When a task expires, its constraints and any grants tied to it end.

## Read next

- [Intent binding](../spec/intent-binding.md)
- [Declaring tasks](../guides/declaring-tasks.md)
