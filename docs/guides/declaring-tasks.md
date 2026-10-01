<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.2 reviewed=2026-10-01 -->

# Declaring tasks

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.2. Nothing on this page works yet.

A [task declaration](../glossary.md#task-declaration) is a structured, human-confirmed statement of what an agent is working on.

## Structure

A task has a description, constraints, and an expiry.

```toml
# task declaration (illustrative)
description = "Fix the login bug"
scope.paths = ["./src/auth/**", "./tests/auth/**"]
forbid.action_types = ["db.schema_change"]
expires_at = "2026-09-14T18:00:00Z"
```

## Constraints

| Constraint | Meaning |
|---|---|
| `scope.paths` | Paths the agent may touch. |
| `forbid.action_types` | Action types the agent may not perform. |
| `allow.hosts` | Hosts the agent may contact. |
| `limits` | Counters for actions such as deletions. |
| `expires_at` | When the task ends. |
| `require_pending_for` | Action types that always need approval. |

## Lifecycle

A task is confirmed by a human, constrains actions while active, and expires.
When a task expires, its constraints and any grants tied to it end.

## Read next

- [Tasks and intent](../concepts/tasks-and-intent.md)
- [Intent binding](../spec/intent-binding.md)