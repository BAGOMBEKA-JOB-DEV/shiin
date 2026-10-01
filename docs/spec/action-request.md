<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Action request

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.action-request/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

An [action request](../glossary.md#action-request) is the normalized, host-independent description of one proposed [action](../glossary.md#action).
An [adapter](../glossary.md#adapter) builds it and the [engine](../glossary.md#engine) evaluates it.

## Fields

**[AR-001]** An action request MUST be a JSON object with the following top-level fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `schema` | string | yes | The schema identifier, `shiin.action-request/v1-draft.1`. |
| `id` | string (UUID) | yes | A unique identifier for this request. |
| `session` | string | yes | The host's session identifier. |
| `agent` | object | yes | The [agent identity](#agent-identity). |
| `action_type` | string | yes | An [action type](action-types.md), such as `fs.write`. |
| `effects` | array of effect | yes | The [effects](#effects) of the action. |
| `task` | string or null | no | The active [task declaration](../glossary.md#task-declaration) id, or null. |
| `policy_snapshot` | string | yes | The digest of the [policy snapshot](../glossary.md#policy-snapshot) used. |
| `host` | object | yes | Host-specific context. |

**[AR-002]** `id` MUST be a version 4 UUID.

## Agent identity

**[AR-003]** The `agent` object MUST contain:

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | yes | The agent identifier. |
| `assurance` | string | yes | The [assurance level](../glossary.md#assurance-level): `asserted`, `attested-local`, `attested-os-user`, or `attested-crypto`. |

## Effects

**[AR-004]** Each element of `effects` MUST be a JSON object with:

| Field | Type | Required | Description |
|---|---|---|---|
| `kind` | string | yes | A free-form label for the effect, such as `path` or `host`. |
| `value` | string | yes | The concrete value, such as a canonical path or host name. |
| `confidence` | string | yes | `exact`, `inferred`, or `opaque`. |

**[AR-005]** An effect with `confidence: "opaque"` MUST indicate that Shiin cannot determine the effect before the action runs.

**[AR-006]** An effect with `confidence: "exact"` MUST be a value Shiin has verified, such as a canonicalized path.

**[AR-007]** An effect with `confidence: "inferred"` MUST be a value Shiin believes but has not verified.

## Host context

**[AR-008]** The `host` object MUST contain:

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | The host name, such as `claude-code`. |
| `tool` | string | no | The host's tool name, if applicable. |
| `payload` | object or null | no | The raw host payload, redacted of secrets. |

## Examples

<!-- validate: spec/schemas/action-request.v1.schema.json -->
```json
{
  "schema": "shiin.action-request/v1-draft.1",
  "id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b",
  "session": "sess-001",
  "agent": {
    "id": "agent-001",
    "assurance": "asserted"
  },
  "action_type": "fs.write",
  "effects": [
    {
      "kind": "path",
      "value": "/home/dev/project/src/login.rs",
      "confidence": "exact"
    }
  ],
  "task": null,
  "policy_snapshot": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "host": {
    "name": "claude-code",
    "tool": "Write",
    "payload": null
  }
}
```

<!-- validate-fail: spec/schemas/action-request.v1.schema.json -->
```json
{
  "schema": "shiin.action-request/v1-draft.1",
  "id": "not-a-uuid",
  "session": "sess-001",
  "agent": {
    "id": "agent-001",
    "assurance": "asserted"
  },
  "action_type": "fs.write",
  "effects": [],
  "task": null,
  "policy_snapshot": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "host": {
    "name": "claude-code"
  }
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.