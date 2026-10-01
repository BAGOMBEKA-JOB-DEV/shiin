<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Audit and evidence

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

## Audit log

The [audit log](../glossary.md#audit-log) is an append-only JSON Lines file of [audit events](../spec/audit-events.md) linked by a hash chain.
It is tamper-evident, not tamper-proof.

## Evidence

Every decision produces an audit event.
The audit log records every request, decision, approval, and grant.

## Verification

`shiin verify` checks that the log's hash chain is intact.

## Replay

`shiin replay` re-evaluates past requests, including against a proposed new policy, to show what would have changed.

## Receipts

From v0.4, signed receipts attest that a decision was made.
A receipt proves who signed a statement about a decision, not that the action's real outcome was correct.

## Read next

- [Audit events](../spec/audit-events.md)
- [Receipts](../spec/receipts.md)
- [Audit, replay, and receipts](../guides/audit-replay-and-receipts.md)