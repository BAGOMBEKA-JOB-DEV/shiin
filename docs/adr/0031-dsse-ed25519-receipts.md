<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0031: DSSE and Ed25519 receipts

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Receipts](../spec/receipts.md), [Cryptography](../security/cryptography.md), [ADR-0032](0032-signing-key-storage.md)

## Decision outcome

A receipt is a signed statement that attests a decision was made.
It uses a DSSE envelope and an Ed25519 signature.
Verification proves who signed a statement about a decision, not that the action's real outcome was correct.

## Consequences
- Receipts verify offline on every Tier 1 platform.
- Key management is separate from signing.
