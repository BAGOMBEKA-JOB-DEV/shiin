<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.3 reviewed=2026-10-01 -->

# Approval protocol

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.approval-protocol/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

An [approval request](../glossary.md#approval-request) is a [pending](../glossary.md#pending) decision waiting for an [approver](../glossary.md#approver) to resolve it ([ADR-0027](../adr/0027-approval-resolutions-and-grants.md), [ADR-0028](../adr/0028-deny-with-ticket-for-non-waiting-hosts.md)).

## Approval request

**[APR-001]** An approval request MUST be created when the decision is `pending` and no matching grant covers it.

**[APR-002]** An approval request MUST carry the action request id, the decision, the reason code, and the agent-facing message.

**[APR-003]** An approval request MUST show the normalized action, not the agent's description of it.

## Resolution

**[APR-004]** A resolution MUST be one of `allow_once`, `allow_always`, or `deny`.

**[APR-005]** `allow_once` creates a single-use [grant](../glossary.md#grant) bound to the digest of one action request.

**[APR-006]** `allow_always` creates a scoped grant covering an agent, action types, a resource pattern, and a workspace.

**[APR-007]** A grant MUST expire.

**[APR-008]** A grant MUST never override `deny`.

## Approver

**[APR-009]** An approver MUST be a human authorized to resolve approval requests.

**[APR-010]** An approver MUST NOT be able to approve through an intercepted channel that the agent controls.

## Deny-with-ticket

**[APR-011]** For a host that cannot wait, the adapter MUST deny the action now and return an approval ticket.

**[APR-012]** If the agent retries after an approver approves, the retry MUST consume a single-use grant.

## Lifecycle

**[APR-013]** An approval request that is never resolved MUST expire as `deny`.

**[APR-014]** An approval request MUST be revocable before it is resolved.

## Example

<!-- validate: spec/schemas/approval-protocol.v1.schema.json -->
```json
{
  "schema": "shiin.approval-protocol/v1-draft.1",
  "request_id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b",
  "decision": "pending",
  "reason": "pending.opaque",
  "message": "The command's effects are unknown.",
  "ticket": "ticket-001",
  "created_at": "2026-10-01T13:58:33Z",
  "expires_at": "2026-10-01T14:03:33Z",
  "resolution": null
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.