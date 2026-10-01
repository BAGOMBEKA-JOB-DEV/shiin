<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0032: Signing key storage

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Receipts](../spec/receipts.md), [Cryptography](../security/cryptography.md), [ADR-0031](0031-dsse-ed25519-receipts.md)

## Decision outcome

Signing keys are stored with restricted permissions in Shiin's state directory.
Keys are rotated on a schedule and on maintainer change.
Old keys are published and retained.

## Consequences
- Keys are not committed to the repository.
- Rotation is tested.
