<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Audit events

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.audit-events/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

An [audit event](../glossary.md#audit-event) is one record in the [audit log](../glossary.md#audit-log) ([ADR-0029](../adr/0029-hash-chained-jsonl-audit-log.md), [ADR-0030](../adr/0030-audit-durability.md)).

## Audit log

**[AUD-001]** The audit log MUST be an append-only JSON Lines file.

**[AUD-002]** Each event MUST be a JSON object on one line.

**[AUD-003]** Events MUST be linked by a [hash chain](../glossary.md#hash-chain): each event includes the hash of the event before it.

## Event kinds

**[AUD-004]** The event kinds are:

| Kind | Meaning |
|---|---|
| `request.received` | An action request was received. |
| `decision.made` | A decision was produced. |
| `approval.requested` | An approval request was created. |
| `approval.resolved` | An approval was resolved. |
| `grant.used` | A grant was consumed. |
| `grant.created` | A grant was created. |
| `grant.revoked` | A grant was revoked. |

## Required fields

**[AUD-005]** Every event MUST carry:

| Field | Type | Description |
|---|---|---|
| `schema` | string | `shiin.audit-event/v1-draft.1`. |
| `id` | string (UUID) | The event id. |
| `timestamp` | string (ISO 8601) | When the event occurred. |
| `kind` | string | The event kind. |
| `prev` | string | The digest of the previous event, or null for the first. |
| `self` | string | The digest of this event. |

## Redaction

**[AUD-006]** Events MUST NOT contain secret values such as tokens, key material, file contents, or environment variables.

**[AUD-007]** Paths, keys, identifiers, and digests are allowed.

## Durability

**[AUD-008]** A non-read action MUST NOT proceed until its audit event is durably recorded.

## Example

<!-- validate: spec/schemas/audit-events.v1.schema.json -->
```json
{
  "schema": "shiin.audit-event/v1-draft.1",
  "id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0c",
  "timestamp": "2026-10-01T13:58:33Z",
  "kind": "decision.made",
  "prev": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "self": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
  "request_id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b",
  "decision": "deny",
  "reason": "deny.policy"
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.