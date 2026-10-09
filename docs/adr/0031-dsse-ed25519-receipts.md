<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0031: DSSE and Ed25519 receipts

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Receipts](../spec/receipts.md), [Glossary](../glossary.md#receipt), [Glossary](../glossary.md#checkpoint), [Glossary](../glossary.md#receipt-verification), [Use case UC-06](../overview/use-cases.md#uc-06-review-evidence-afterward), [ADR-0029](0029-hash-chained-jsonl-audit-log.md)

## Context and problem statement

Hash chaining makes the audit log tamper-evident, but an attacker who can rewrite the whole file can also rewrite the hashes.
The project needed a way to produce signed attestations that a decision was made, so that an independent observer can verify the log's integrity without trusting the host.

## Decision drivers

- Signatures must be verifiable by anyone with the operator's public key.
- The format must be standard, not a custom invention.
- The signing key must be stored securely (see [ADR-0032](0032-signing-key-storage.md)).

## Considered options

1. DSSE envelopes with Ed25519 signatures for checkpoints and receipts.
2. A custom signature format wrapping the decision JSON.
3. X.509 certificates with RSA signatures.

## Decision outcome

Chosen option: "DSSE envelopes with Ed25519 signatures", because DSSE is an emerging standard for software supply-chain attestations and Ed25519 is fast and widely supported.

- Checkpoints are signed DSSE envelopes containing the audit log's head hash and position.
- Receipts are signed DSSE envelopes attesting that a specific decision was made.
- Signatures use Ed25519.
- `shiin verify` checks signatures and the hash chain offline.
- `shiin replay` re-evaluates recorded requests against a policy snapshot, including a different policy, to show what would have changed.

### Consequences

- Good, because DSSE and Ed25519 are standards with libraries in many languages.
- Good, because offline verification does not require network access.
- Bad, because it adds a signing-key management requirement.

### Confirmation

- The [receipts specification](../spec/receipts.md) defines the DSSE envelope structure.
- Receipts verify offline on every Tier 1 platform (exit criterion: v0.4).

## Pros and cons of the options

### DSSE with Ed25519

- Good, because it uses standard, audited formats.
- Good, because Ed25519 is fast and compact.
- Bad, because it requires key management.

### Custom signature format

- Good, because it avoids external dependencies.
- Bad, because custom crypto is dangerous and not independently verifiable.

### X.509 with RSA

- Good, because enterprises standardize on X.509.
- Bad, because it is heavier and not offline-friendly.

## More information

The [signing key storage ADR](0032-signing-key-storage.md) defines where keys live. The cryptography page describes the hash functions used in the chain.
