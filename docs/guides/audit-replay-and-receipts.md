<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.4 reviewed=2026-10-01 -->

# Audit, replay, and receipts

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.4. Nothing on this page works yet.

## Audit log

The [audit log](../glossary.md#audit-log) is an append-only JSON Lines file of [audit events](../spec/audit-events.md) linked by a hash chain.
It is tamper-evident, not tamper-proof.

## Verify

`shiin verify` checks that the log's hash chain is intact.

## Replay

`shiin replay` re-evaluates past requests against their recorded [policy snapshots](../glossary.md#policy-snapshot).
It can replay against a different policy to answer "what if".

## Receipts

Signed [receipts](../spec/receipts.md) attest that a decision was made.
A receipt proves who signed a statement about a decision, not that the action's real outcome was correct.

## Read next

- [Audit events](../spec/audit-events.md)
- [Receipts](../spec/receipts.md)
- [Audit and evidence](../concepts/audit-and-evidence.md)