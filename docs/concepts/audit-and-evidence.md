<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-15 -->

# Audit and evidence

> [!NOTE]
> **Guide: draft.**
> The source build implements the current behavior below; remaining design work is marked explicitly.

## Audit log

The [audit log](../glossary.md#audit-log) is an append-only JSON Lines file of [audit events](../spec/audit-events.md) linked by a hash chain.
It is tamper-evident, not tamper-proof.

## Evidence

Hook requests and decisions produce audit events before the hook responds.
Offline `check` and `explain` do not write hook audit events.
Approval and grant events are planned for v0.3.
Command and URL values are recorded as digests, as described in [audit events](../spec/audit-events.md).

## Verification

`shiin verify` checks that the log's hash chain is intact.

## Replay

Planned for v0.4: `shiin replay` re-evaluates past requests, including against a proposed new policy, to show what would have changed.

## Receipts

From v0.4, signed receipts attest that a decision was made.
A receipt proves who signed a statement about a decision, not that the action's real outcome was correct.

## Read next

- [Audit events](../spec/audit-events.md)
- [Receipts](../spec/receipts.md)
- [Audit, replay, and receipts](../guides/audit-replay-and-receipts.md)
