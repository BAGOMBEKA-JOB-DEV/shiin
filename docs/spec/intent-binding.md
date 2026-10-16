<!-- shiin-doc: kind=spec status=draft implementation=partial milestone=v0.2 reviewed=2026-10-15 -->

# Intent binding

> [!NOTE]
> **Specification: draft.**
> Implemented in part in the source build. Remaining requirements and external validation are not complete.
>
> Version: `shiin.intent-binding/v1-draft.2`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

[Intent binding](../glossary.md#intent-binding) checks an [action](../glossary.md#action) against the [constraints](../glossary.md#constraint) of the active [task declaration](../glossary.md#task-declaration) ([ADR-0020](../adr/0020-structured-intent-binding.md)).

## Task declaration

**[INT-001]** A task declaration MUST be a structured, human-confirmed statement with a description, constraints, and an expiry.

**[INT-002]** A task declaration MUST have a unique id.

**[INT-003]** The description is recorded but never evaluated.

## Constraints

**[INT-004]** The constraint set is closed. Only these kinds are allowed:

| Constraint | Meaning |
|---|---|
| `scope.paths` | Paths the agent may touch. |
| `forbid.action_types` | Action types the agent may not perform. |
| `allow.hosts` | Hosts the agent may contact. |
| `limits` | Counters for actions such as deletions. |
| `expires_at` | When the task ends. |
| `require_pending_for` | Action types that always need approval. |

**[INT-005]** A constraint MUST be machine-checkable.

## Behavior

**[INT-006]** The intent stage is a [gate](../glossary.md#gate): it may return `deny`, `pending`, or `pass`, but never `allow`.

**[INT-007]** An action outside `scope.paths` MUST produce `deny` with reason code `deny.intent`.

**[INT-008]** An action in `forbid.action_types` MUST produce `deny` with reason code `deny.intent`.

**[INT-009]** An action exceeding a `limits` counter MUST produce `deny` with reason code `deny.intent`.

**[INT-010]** An action in `require_pending_for` MUST produce `pending` with reason code `pending.intent`.

**[INT-011]** When no task is active, the intent stage MUST produce `pass`.

## Expiry

**[INT-012]** An expired task MUST no longer constrain actions.

**[INT-013]** Grants tied to an expired task MUST end.

## Example

<!-- validate: spec/schemas/intent-binding.v1.schema.json -->
```json
{
  "schema": "shiin.intent-binding/v1-draft.1",
  "task": {
    "id": "task-001",
    "description": "Fix the login bug",
    "scope": {
      "paths": ["./src/auth/**", "./tests/auth/**"]
    },
    "forbid": {
      "action_types": ["db.schema_change"]
    },
    "expires_at": "2026-09-14T18:00:00Z"
  },
  "action": {
    "action_type": "fs.write",
    "effects": [
      { "kind": "path", "value": "/home/dev/project/migrations/0042.sql", "confidence": "exact" }
    ]
  },
  "result": "deny",
  "reason": "deny.intent"
}
```

## Evaluation plan

The first evaluation uses every published conformance vector that carries a task declaration.
Each case is evaluated with its recorded task, counters, mode, and time, then evaluated again with the task removed.
The report records the expected and actual decision/reason, changes from the policy-only baseline, false allows against non-allow labels, and false denials against allow labels.
Any disagreement with a labelled case fails the command.

```console
$ cargo xtask intent-evaluation
```

The documentation CI job runs this measurement and keeps its report in the job log.
The [2026-10-16 hosted measurement](https://github.com/BAGOMBEKA-JOB-DEV/shiin/actions/runs/38876644402/job/116893396635) matched all nine task cases, including their reason codes.
Eight outcomes changed from the policy-only baseline; the corpus recorded zero false allows among eight non-allow labels and zero false denials among one allow label.
The small specification corpus establishes reproducible behavior on the declared constraints, not real-world false-positive or false-negative rates.
The next evaluation requires independently labelled task/action sequences, named host versions, interception results, and operator review of unnecessary holds and missed task violations.
Task declarations remain experimental until that broader evaluation is available.

## Revision history

- v1-draft.2 (2026-10-12): The schema now carries every constraint kind the closed set names. `allow.hosts`, `limits`, and `require_pending_for` were described in the text but missing from the schema, so a task that used them failed validation.
- v1-draft.1 (2026-10-01): Initial draft.
