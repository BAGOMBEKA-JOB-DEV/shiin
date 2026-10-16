<!-- shiin-doc: kind=spec status=draft implementation=partial milestone=v0.4 reviewed=2026-10-16 -->

# Receipts

> [!NOTE]
> **Specification: draft.**
> Normative design. Signed receipts, checkpoints, and encrypted replay artifacts are implemented in source; schema and cross-platform conformance remain incomplete.
>
> Version: `shiin.receipts/v1-draft.3`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

A [receipt](../glossary.md#receipt) is a signed statement that attests a [decision](../glossary.md#decision) was made ([ADR-0031](../adr/0031-dsse-ed25519-receipts.md), [ADR-0032](../adr/0032-signing-key-storage.md)).

## Envelope

**[RCPT-001]** A receipt MUST use a DSSE (Dead Simple Signing Envelope) envelope.

**[RCPT-002]** A receipt MUST be signed with Ed25519.

## Payload

**[RCPT-003]** The payload MUST include the action request id, the decision, the reason code, and the policy snapshot digest.

**[RCPT-004]** The payload MUST include the signing key identifier.

## Verification

**[RCPT-005]** [Receipt verification](../glossary.md#receipt-verification) checks the signature and contents.

**[RCPT-006]** Verification proves who signed a statement about a decision, not that the action's real outcome was correct.

## Key management

**[RCPT-007]** Signing keys MUST be stored as described in [Signing key storage](../adr/0032-signing-key-storage.md).

**[RCPT-008]** A decision receipt MUST include the SHA-256 digest of the canonical action request whose decision it attests.

**[RCPT-009]** A decision receipt and audit checkpoint MUST include their signing time as an RFC 3339 timestamp.

## Checkpoints

A checkpoint signs a position in the audit log so an observer can compare a local history with a trusted head.

**[RCPT-010]** A checkpoint MUST include the unsigned sequence number, the SHA-256 digest at that position, the signing key identifier, and its signing time.

The checkpoint's sequence number is one-based for audit events; sequence zero identifies the genesis digest.

## Encrypted replay inputs

Replay artifacts preserve the inputs needed to re-evaluate one decision.
They are encrypted locally and signed so replay requires the originating replay key while authenticity can be checked before decryption.

**[RCPT-011]** A replay artifact MUST encrypt the recorded request, policy, task, usage, evaluation time, mode, decision, grants, and engine version using XChaCha20-Poly1305 with a fresh 192-bit nonce.

**[RCPT-012]** Replay decryption MUST bind the replay key identifier and request identifier as authenticated associated data.

## Current implementation

The daemon writes signed decision receipts, encrypted replay inputs, and a signed audit checkpoint under `.shiin/evidence/` when it starts with `SHIIN_EVIDENCE=1` and an initialized signing key.
`shiin verify <evidence.json> <public.json>` verifies a receipt offline.
For a checkpoint, it also checks the matching local `.shiin/audit.jsonl` history, so copying only the checkpoint and public key to another machine is insufficient.
`shiin replay <encrypted-replay.json>` requires the matching local replay key; the optional policy argument performs a what-if comparison.
Windows support, conformance vectors for all evidence payloads, and Tier 1 offline verification remain open.

## Checkpoint example

<!-- validate: spec/schemas/checkpoint.v1.schema.json -->
```json
{
  "schema": "shiin.checkpoint/v1-draft.1",
  "key_id": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "seq": 1,
  "head": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "signed_at": "2026-10-16T13:58:33Z"
}
```

## Encrypted replay example

<!-- validate: spec/schemas/encrypted-replay.v1.schema.json -->
```json
{
  "schema": "shiin.replay-encrypted/v1-draft.1",
  "key_id": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "request_id": "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b",
  "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
  "ciphertext": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
}
```

## Example

<!-- validate: spec/schemas/receipts.v1.schema.json -->
```json
{
  "schema": "shiin.receipt/v1-draft.3",
  "key_id": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "request_id": "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b",
  "decision": "allow",
  "reason": "allow.policy",
  "policy_snapshot": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "request_digest": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
  "signed_at": "2026-10-01T13:58:33Z"
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.
- v1-draft.2 (2026-10-16): Add the request digest bound by the source implementation.
- v1-draft.3 (2026-10-16): Specify signed checkpoints and encrypted replay claims and their validation fields.
