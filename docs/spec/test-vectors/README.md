<!-- shiin-doc: kind=spec status=draft implementation=complete milestone=m0 reviewed=2026-10-12 -->

# Test vectors

> [!NOTE]
> **Specification: draft.**
> The engine conformance test and `shiin policy test` execute every vector.

Test vectors encode the [golden use cases](../../overview/use-cases.md) and are the conformance corpus that implementations are checked against.

## Format

Each test vector is a JSON file in `docs/spec/test-vectors/` validated against `test-vector.v1.schema.json`.

A test vector records one action request, the policy it is evaluated against, and the expected decision.

Optional fields carry the remaining engine inputs, so that every vector can be decided from the vector alone:

| Field | Meaning |
|---|---|
| `task` | The active task declaration, for vectors that exercise intent binding. |
| `mode` | `attended` or `unattended`. Absent means attended. |
| `usage` | What the task has used before this action, for vectors that exercise limits. |
| `now` | The evaluation time, for vectors that exercise a task expiry. |

## Golden use cases

| ID | Use case | Vector file |
|---|---|---|
| TV-001 | An allowed edit | `tv-001-allowed-edit.v1.json` |
| TV-002 | A denied read of a secret | `tv-002-denied-secret.v1.json` |
| TV-003 | A command with unknown effects | `tv-003-opaque-command.v1.json` |
| TV-004 | An action outside the task | `tv-004-outside-task.v1.json` |
| TV-005 | A bulk deletion | `tv-005-bulk-delete.v1.json` |
| TV-006 | A destructive VCS operation | `tv-006-destructive-vcs.v1.json` |
| TV-007 | An unattended deny | `tv-007-unattended-deny.v1.json` |
| TV-008 | No rule matches, so the default decision applies | `tv-008-default-deny.v1.json` |
| TV-009 | A secret read through a shell command, denied by the same rule as a direct read | `tv-009-secret-via-shell.v1.json` |
| TV-010 | A task's write limit is reached | `tv-010-task-limit.v1.json` |
| TV-011 | An expired task no longer constrains | `tv-011-task-expired.v1.json` |
| TV-012 | A task asks a person about an action type | `tv-012-task-asks.v1.json` |
| TV-013 | A task limits which hosts may be reached | `tv-013-task-host.v1.json` |
| TV-014 | Task scope denial wins over required approval | `tv-014-task-deny-before-pending.v1.json` |
| TV-015 | A task forbids a read performed through a shell command | `tv-015-task-forbidden-effect.v1.json` |
| TV-016 | Workspace-relative task scope excludes another workspace | `tv-016-task-scope-root.v1.json` |
| TV-017 | A spent task budget wins over required approval | `tv-017-task-budget-before-pending.v1.json` |
| TV-018 | Outside-workspace risk denies an otherwise allowed write | `tv-018-outside-workspace-risk.v1.json` |
| TV-019 | Secret-path risk denies an otherwise allowed read | `tv-019-secret-path-risk.v1.json` |

## Running

Implementations run every vector and check the decision matches.

## Revision history

- v1-draft.5 (2026-10-12): Added TV-014 to TV-019 for task denial precedence, forbidden effects, workspace-relative scope boundaries, and configured path risk signals.
- v1-draft.4 (2026-10-12): Added TV-010 to TV-013 for task limits, expiry, ask-for, and host constraints, and the `usage` and `now` fields they need.
- v1-draft.3 (2026-10-12): Added TV-009, which pins UC-02: a rule written for `fs.read` must also cover a read performed by a shell command.
- v1-draft.2 (2026-10-12): Added TV-008 for the default decision, a workspace root to every request, and the optional `task` and `mode` fields, so TV-004 and TV-007 are decidable from their own inputs.
- v1-draft.1 (2026-10-01): Initial draft.
