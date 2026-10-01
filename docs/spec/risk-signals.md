<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Risk signals

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.risk-signals/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

A [risk signal](../glossary.md#risk-signal) is a discrete, deterministic property of an [action request](action-request.md) that can escalate a decision ([ADR-0021](../adr/0021-discrete-risk-signals.md)).
Shiin does not use numeric risk scores.

## Behavior

**[RISK-001]** The risk stage is a [gate](../glossary.md#gate): it may return `deny`, `pending`, or `pass`, but never `allow`.

**[RISK-002]** A signal is triggered by matching a property of the action request.

**[RISK-003]** A triggered signal produces the decision configured for it: `deny`, `pending`, or `pass`.

## Signals

### `secret_path`

**[RISK-004]** `secret_path` is triggered when an effect touches a path known to hold secrets, such as `**/.env`, `**/.env.*`, `~/.ssh/**`, or cloud credential files.

**[RISK-005]** `secret_path` is configured in the policy `risk` table.

### `outside_workspace`

**[RISK-006]** `outside_workspace` is triggered when an effect touches a path outside the workspace root.

### `bulk_delete`

**[RISK-007]** `bulk_delete` is triggered when a single action would delete more files than a configured threshold.

### `opaque_command`

**[RISK-008]** `opaque_command` is triggered when an action carries an [opaque effect](../glossary.md#opaque-effect).

### `destructive_vcs`

**[RISK-009]** `destructive_vcs` is triggered by version-control operations that can destroy shared history, such as `git push --force`.

## Configuration

**[RISK-010]** Every signal MUST have a configured decision: `deny`, `pending`, or `pass`.

**[RISK-011]** A signal that is not configured MUST produce `pass`.

## Examples

<!-- validate: spec/schemas/risk-signals.v1.schema.json -->
```json
{
  "schema": "shiin.risk-signals/v1-draft.1",
  "signals": [
    {
      "name": "secret_path",
      "triggered": true,
      "decision": "deny",
      "reason": "deny.risk"
    },
    {
      "name": "bulk_delete",
      "triggered": false,
      "decision": "pending",
      "reason": null
    }
  ]
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.