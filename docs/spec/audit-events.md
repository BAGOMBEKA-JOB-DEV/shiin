<!-- shiin-doc: kind=spec status=draft implementation=partial milestone=v0.4 reviewed=2026-10-16 -->

# Audit events

> [!NOTE]
> **Specification: draft.**
> Requests, decisions, approval and grant events, hash chaining, redacted summaries, and local verification exist in source.
> Signed evidence and replay are implemented in part; platform and conformance validation remain open.
>
> Version: `shiin.audit-events/v1-draft.2`.

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
| `schema` | string | `shiin.audit-event/v1-draft.2`. |
| `id` | string (UUID) | The event id. |
| `timestamp` | string (ISO 8601) | When the event occurred. |
| `kind` | string | The event kind. |
| `prev` | string | The digest of the previous event; the first uses `sha256:` followed by 64 zeroes. |
| `self` | string | The digest of this event. |
| `seq` | integer | Position in the log, beginning at 1. |
| `payload` | object | An allowlisted summary and the SHA-256 digest of the original normalized body. |

The hash is SHA-256 of the compact UTF-8 JSON event with `self` omitted and object keys sorted lexicographically at every depth.
Array order is preserved.
The writer verifies the existing chain before extending it and syncs every appended event.

Draft.1 logs emitted before this reconciliation used a different, nonconforming shape.
They are not silently accepted or migrated.
Preserve them separately before starting a new log; do not delete evidence.

## Redaction

**[AUD-006]** Events MUST NOT contain secret values such as tokens, key material, file contents, or environment variables.

**[AUD-007]** Paths, keys, identifiers, and digests are allowed.

Current summaries retain request, agent, session, task, host, workspace, decision, and snapshot identifiers, plus path and count effects.
Command, URL, and other effect values are replaced with `value_digest`.
File contents, raw host payloads, free-text messages, and unknown fields are omitted.
The resulting summary is not a complete action request.
An audit summary alone cannot support exact replay because sensitive effect values are redacted. The daemon stores replay inputs separately in encrypted evidence; replay and its key path have a source implementation, with platform and external review still open.

## Durability

**[AUD-008]** A non-read action MUST NOT proceed until its audit event is durably recorded.

## Example

<!-- validate: spec/schemas/audit-events.v1.schema.json -->
```json
{
  "schema": "shiin.audit-event/v1-draft.2",
  "id": "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0c",
  "timestamp": "2026-10-01T13:58:33Z",
  "kind": "decision.made",
  "prev": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "self": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
  "seq": 1,
  "payload": {
    "digest": "sha256:2222222222222222222222222222222222222222222222222222222222222222"
  },
  "request_id": "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b",
  "decision": "deny",
  "reason": "deny.policy"
}
```

## Revision history

- v1-draft.2 (2026-10-15): Reconciled wire names, sequence numbers, hash construction, and redacted summaries; explicitly reject incompatible earlier logs.
- v1-draft.1 (2026-10-01): Initial draft.
