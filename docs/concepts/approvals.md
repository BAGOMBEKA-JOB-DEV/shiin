<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.3 reviewed=2026-10-16 -->

# Approvals

> [!NOTE]
> **Guide: draft.**
> Unix approval requests, grants, and CLI resolution exist in source. Remote approvals remain a draft proposal in RFC-0001.

An [approval request](../glossary.md#approval-request) is a [pending](../glossary.md#pending) decision waiting for an [approver](../glossary.md#approver) to resolve it.

## Resolution

An approver can:

- **Allow once.** Creates a single-use [grant](../glossary.md#grant) bound to the digest of one action request.
- **Allow for a scope and time.** Creates a scoped grant covering an agent, action types, a resource pattern, and a workspace, which expires.
- **Deny.** The action is refused.

A grant never overrides `deny`.

## Channels

Approval requests reach a human through an [approval channel](../glossary.md#approval-channel).
The current source implementation supports a separate operator terminal on Unix.
Remote approval is not implemented; RFC-0001 proposes an opt-in signed polling transport.

## Deny-with-ticket

For a host that cannot wait, the adapter denies the action now and returns an approval ticket.
If the agent retries after an approver approves, the retry consumes a single-use grant.

## Expiry

An approval request that is never resolved expires as `deny`.
It never becomes `allow` on its own.

## Read next

- [Approval protocol](../spec/approval-protocol.md)
- [Handling approvals](../guides/handling-approvals.md)
