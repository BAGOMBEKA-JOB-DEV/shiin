<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-15 -->

# Risk signals

> [!NOTE]
> **Guide: draft.**
> The source build implements the current behavior below; remaining design work is marked explicitly.

A [risk signal](../glossary.md#risk-signal) is a discrete, deterministic property of an [action request](../glossary.md#action-request) that can escalate a decision.
Shiin does not use numeric risk scores.

## Signals

| Signal | Triggered by |
|---|---|
| `secret_path` | An effect touches a path known to hold secrets. |
| `outside_workspace` | An effect touches a path outside the workspace. |
| `bulk_delete` | A single action would delete more files than a threshold. |
| `opaque_command` | An action carries an opaque effect. |
| `destructive_vcs` | A version-control operation that can destroy shared history. |

## Configuration

Each signal has a configured decision: `deny`, `pending`, or `pass`.
A signal that is not configured produces `pass`.

## Read next

- [Risk signals](../spec/risk-signals.md)
- [Evaluation](../spec/evaluation.md)
