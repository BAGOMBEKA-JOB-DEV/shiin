<!-- shiin-doc: kind=spec status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# Test vectors

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.

Test vectors encode the [golden use cases](../overview/use-cases.md) and are the conformance corpus that implementations are checked against.

## Format

Each test vector is a JSON file in `docs/spec/test-vectors/` validated against `test-vector.v1.schema.json`.

A test vector records one action request, the policy snapshot it is evaluated against, and the expected decision.

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

## Running

Implementations run every vector and check the decision matches.

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.