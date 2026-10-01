<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.4 reviewed=2026-10-01 -->

# Receipts

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.receipts/v1-draft.1`.

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

## Example

<!-- validate: spec/schemas/receipts.v1.schema.json -->
```json
{
  "schema": "shiin.receipt/v1-draft.1",
  "key_id": "key-001",
  "request_id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b",
  "decision": "allow",
  "reason": "allow.policy",
  "policy_snapshot": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "signed_at": "2026-10-01T13:58:33Z",
  "signature": "sig-001"
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.