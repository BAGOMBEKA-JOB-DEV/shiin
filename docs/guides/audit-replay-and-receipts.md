<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.4 reviewed=2026-10-16 -->

# Audit, replay, and receipts

> [!NOTE]
> **Guide: draft.**
> The source daemon can produce signed receipts, checkpoints, and encrypted replay inputs. The workflows below require a source build and have platform and conformance limits.

## Audit log

The [audit log](../glossary.md#audit-log) is an append-only JSON Lines file of [audit events](../spec/audit-events.md) linked by a hash chain.
It is tamper-evident, not tamper-proof.

## Verify

`shiin verify` checks that the current workspace's log hash chain is intact.
To verify a signed decision receipt against a trusted public key, run `shiin verify <receipt.json> <public.json>`.
Signed evidence is written under `.shiin/evidence/` when the daemon starts with `SHIIN_EVIDENCE=1` after `shiin key init` has initialized the OS keyring.
Checkpoint verification also compares the signed head with the current local audit log, so it needs the matching history.

## Replay

`shiin replay <encrypted-replay.json>` re-evaluates a recorded request against its captured policy and inputs.
It requires the same OS keyring that encrypted the replay data.
Pass a policy path as a second argument to compare the decision under that policy.

## Receipts

Signed [receipts](../spec/receipts.md) attest that a decision was made.
A receipt proves who signed a statement about a decision, not that the action's real outcome was correct.
Evidence support remains partial: Windows key storage, complete payload schemas and vectors, and Tier 1 offline verification have not been validated.

## Read next

- [Audit events](../spec/audit-events.md)
- [Receipts](../spec/receipts.md)
- [Audit and evidence](../concepts/audit-and-evidence.md)
