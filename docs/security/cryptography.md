<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Cryptography

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's cryptography rules ([ADR-0031](../adr/0031-dsse-ed25519-receipts.md), [ADR-0032](../adr/0032-signing-key-storage.md)).

## Algorithms

| Use | Algorithm |
|---|---|
| Receipt signatures | Ed25519 |
| Audit hash chain | SHA-256 |
| Receipt envelope | DSSE |

## Key management

Signing keys are stored with restricted permissions in Shiin's state directory.
Keys are rotated on a schedule and on maintainer change.

## Randomness

Shiin uses the operating system's randomness source for ids and keys.
The engine crates never use a custom randomness source.

## Read next
- [Receipts](../spec/receipts.md)
- [Signing key storage](../adr/0032-signing-key-storage.md)