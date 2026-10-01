<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Decision

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.decision/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

A [decision](../glossary.md#decision) is the [engine](../glossary.md#engine)'s answer to an [action request](action-request.md).

## Decision values

**[DEC-001]** A decision MUST be one of `allow`, `deny`, or `pending`.

**[DEC-002]** `allow` means the action may proceed.

**[DEC-003]** `deny` means the action must not proceed.

**[DEC-004]** `pending` means the action must wait for an [approver](../glossary.md#approver).

## Reason codes

**[DEC-005]** Every decision MUST carry a stable, machine-readable [reason code](../glossary.md#reason-code).

**[DEC-006]** Reason codes MUST follow the form `<decision>.<stage>` or `<decision>.<signal>`, such as `deny.policy` or `pending.opaque`.

**[DEC-007]** The reason codes defined by this specification are:

| Code | Decision | Meaning |
|---|---|---|
| `allow.policy` | allow | A policy rule allowed the action. |
| `allow.grant` | allow | A [grant](../glossary.md#grant) covered the action. |
| `deny.identity` | deny | The agent identity is not acceptable. |
| `deny.capability` | deny | The agent lacks a capability for this action class. |
| `deny.policy` | deny | A policy rule denied the action. |
| `deny.intent` | deny | The action does not fit the active task. |
| `deny.risk` | deny | A risk signal denied the action. |
| `deny.failure` | deny | A failure produced a fail-closed deny. |
| `deny.builtin` | deny | A built-in protected resource was touched. |
| `pending.identity` | pending | The agent identity needs confirmation. |
| `pending.capability` | pending | The agent capability needs confirmation. |
| `pending.intent` | pending | The action needs confirmation against the task. |
| `pending.risk` | pending | A risk signal escalated the action. |
| `pending.opaque` | pending | An opaque effect needs approval. |
| `pending.grantable` | pending | The action needs a grant or approval. |

## Agent-facing message

**[DEC-008]** Every decision MUST carry a message intended for the agent.

**[DEC-009]** The message MUST NOT reveal policy structure, error codes, paths, or other internal detail that could help an agent probe for a bypass.

## Decision object

**[DEC-010]** A decision MUST be a JSON object with:

| Field | Type | Required | Description |
|---|---|---|---|
| `schema` | string | yes | `shiin.decision/v1-draft.1`. |
| `request_id` | string (UUID) | yes | The id of the action request. |
| `decision` | string | yes | `allow`, `deny`, or `pending`. |
| `reason` | string | yes | The reason code. |
| `message` | string | yes | The agent-facing message. |
| `trace` | array | no | The [trace](../glossary.md#trace) of stage results, in explain mode. |
| `approval` | object or null | no | An [approval request](../glossary.md#approval-request), when `pending`. |

## Examples

<!-- validate: spec/schemas/decision.v1.schema.json -->
```json
{
  "schema": "shiin.decision/v1-draft.1",
  "request_id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b",
  "decision": "deny",
  "reason": "deny.policy",
  "message": "Reading secret files is not allowed.",
  "trace": null,
  "approval": null
}
```

<!-- validate-fail: spec/schemas/decision.v1.schema.json -->
```json
{
  "schema": "shiin.decision/v1-draft.1",
  "request_id": "not-a-uuid",
  "decision": "allow",
  "reason": "allow.policy",
  "message": "ok",
  "trace": null,
  "approval": null
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.