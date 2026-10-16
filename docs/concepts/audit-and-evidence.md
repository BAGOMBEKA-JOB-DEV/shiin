<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.4 reviewed=2026-10-16 -->

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
Approval and grant events exist in the Unix daemon's source implementation.
Command and URL values are recorded as digests, as described in [audit events](../spec/audit-events.md).

## Verification

`shiin verify` checks that the log's hash chain is intact.

## Replay

`shiin replay` re-evaluates captured requests with the originating replay key and can compare a different policy.

## Receipts

Signed receipts attest that a decision was made when evidence recording is enabled in the daemon.
A receipt proves who signed a statement about a decision, not that the action's real outcome was correct.

## Read next

- [Audit events](../spec/audit-events.md)
- [Receipts](../spec/receipts.md)
- [Audit, replay, and receipts](../guides/audit-replay-and-receipts.md)
