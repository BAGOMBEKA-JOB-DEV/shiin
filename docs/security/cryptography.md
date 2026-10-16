<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-16 -->

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

Signing key generation, rotation, and public-key export use the OS keyring, following [ADR-0032](../adr/0032-signing-key-storage.md).
Replay encryption keys are stored alongside the signing key in the keyring.
Native key-store lifecycle checks pass on hosted Linux, macOS, and Windows runners, including persistence, rotation, retained-key replay, and corrupt-entry refusal.
The [testing strategy](../engineering/testing-strategy.md#native-key-stores) describes the isolated fixture and its limits.
Live operator-environment qualification and independent review remain open.

## Randomness

Shiin uses the operating system's randomness source for ids and keys.
The engine crates never use a custom randomness source.

## Read next

- [Receipts](../spec/receipts.md)
- [Signing key storage](../adr/0032-signing-key-storage.md)
