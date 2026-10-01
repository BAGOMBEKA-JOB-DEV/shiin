<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.3 reviewed=2026-10-01 -->

# Approvals

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.3. Nothing on this page works yet.

An [approval request](../glossary.md#approval-request) is a [pending](../glossary.md#pending) decision waiting for an [approver](../glossary.md#approver) to resolve it.

## Resolution

An approver can:

- **Allow once.** Creates a single-use [grant](../glossary.md#grant) bound to the digest of one action request.
- **Allow for a scope and time.** Creates a scoped grant covering an agent, action types, a resource pattern, and a workspace, which expires.
- **Deny.** The action is refused.

A grant never overrides `deny`.

## Channels

Approval requests reach approvers through an [approval channel](../glossary.md#approval-channel), such as the command-line approver or a webhook.

## Deny-with-ticket

For a host that cannot wait, the adapter denies the action now and returns an approval ticket.
If the agent retries after an approver approves, the retry consumes a single-use grant.

## Expiry

An approval request that is never resolved expires as `deny`.
It never becomes `allow` on its own.

## Read next

- [Approval protocol](../spec/approval-protocol.md)
- [Handling approvals](../guides/handling-approvals.md)