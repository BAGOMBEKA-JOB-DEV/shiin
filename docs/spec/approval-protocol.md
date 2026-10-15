<!-- shiin-doc: kind=spec status=draft implementation=partial milestone=v0.3 reviewed=2026-10-15 -->

# Approval protocol

> [!NOTE]
> **Specification: draft.**
> Local approval state, grants, revocation, and Unix daemon integration exist in source.
> Host validation and independent security review remain open.
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

## Local implementation

The Unix daemon uses a 0700 per-user directory and a 0600 local socket, checks peer credentials, and holds an exclusive state lock.
An in-memory 256-bit operator credential is disclosed only in an explicitly started operator terminal.
Agent-facing requests cannot administer approvals without that credential.
It is never included in tickets, audit records, files, or environment variables.

The retry binding hashes the normalized request with only its fresh correlation `id` excluded.
Agent, session, action type, every effect, task, workspace, policy snapshot, and host context remain bound.
This lets a host retry the same action with a new request UUID without making the ticket itself an authorization credential.
Single-use consumption is persisted before an allow response.
Scoped grants require an explicit agent, exact action types, a resource glob, and a workspace; session, task, and snapshot remain bound as well.
The initial pending deadline is five minutes, and grant lifetimes are positive and at most 24 hours, capped by an active task's expiry.

The service re-evaluates the current policy and task before consulting grants.
A deny is never eligible for a grant.
State corruption, failed audit writes, failed persistence, missing task counters, and a configured but unreachable daemon all fail closed.

## Example

<!-- validate: spec/schemas/approval-protocol.v1.schema.json -->
```json
{
  "schema": "shiin.approval-protocol/v1-draft.1",
  "request_id": "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b",
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

- Implementation update (2026-10-15): Added local service behavior and the retry binding; the normative draft version is unchanged.
- v1-draft.1 (2026-10-01): Initial draft.
