<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.3 reviewed=2026-10-01 -->

# Handling approvals

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.3. Nothing on this page works yet.

## As an approver

When an action is `pending`, an [approval request](../glossary.md#approval-request) is created.
You can:

- **Allow once.** The action proceeds; the grant is consumed.
- **Allow for a scope and time.** The action and similar ones proceed until the grant expires.
- **Deny.** The action is refused.

## As a developer

If you run an agent and an action is pending, you will be asked to approve it.
You can approve it once or create a scoped grant.

## Deny-with-ticket

For a host that cannot wait, the adapter denies the action now and returns an approval ticket.
If the agent retries after an approver approves, the retry consumes a single-use grant.

## Expiry

An approval request that is never resolved expires as `deny`.

## Read next

- [Approvals](../concepts/approvals.md)
- [Approval protocol](../spec/approval-protocol.md)