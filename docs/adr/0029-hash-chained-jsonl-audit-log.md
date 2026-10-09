<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0029: Hash-chained JSON Lines audit log

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Audit events](../spec/audit-events.md), [Glossary](../glossary.md#audit-log), [Glossary](../glossary.md#hash-chain), [Use case UC-06](../overview/use-cases.md#uc-06-review-evidence-afterward)

## Context and problem statement

Every decision must leave a record that an operator or auditor can review afterward.
The audit log must be append-only and tamper-evident: if an event is removed or altered, the tampering must be detectable.
The format must be simple, streamable, and compatible with existing log-processing tools.

## Decision drivers

- Tamper evidence: altering past events must be detectable.
- Streamable: events can be appended and read incrementally.
- Tool-compatible: existing JSON Lines tools can process the log.

## Considered options

1. JSON Lines where each event includes the hash of the previous event.
2. A binary append-only log with fixed-size records.
3. A database-backed log with transactional guarantees.

## Decision outcome

Chosen option: "JSON Lines with hash chaining", because it provides tamper evidence while remaining streamable and compatible with standard tooling.

- The audit log is an append-only JSON Lines file.
- Each audit event includes the hash of the previous event's serialization, forming a chain.
- Removing or altering an event breaks the chain and is detectable by recomputing hashes.
- The log is tamper-evident, not tamper-proof: an attacker with write access can replace the whole file.

### Consequences

- Good, because the format is human-readable and streamable.
- Good, because standard JSON tools can process it.
- Bad, because hash chaining provides no protection against an attacker who can rewrite the whole file; that requires signed checkpoints ([ADR-0031](0031-dsse-ed25519-receipts.md)).

### Confirmation

- The [audit events specification](../spec/audit-events.md) defines the event schema and the hash-chain fields.
- `shiin verify` checks the hash chain and reports breakage.

## Pros and cons of the options

### JSON Lines with hash chaining

- Good, because it is streamable, readable, and tamper-evident.
- Bad, because it does not prevent full-file replacement.

### Binary fixed-size records

- Good, because it is efficient.
- Bad, because it is not human-readable and requires custom tools.

### Database-backed log

- Good, because it has transactional guarantees.
- Bad, because it adds a database dependency and is harder to audit independently.

## More information

Signed checkpoints and receipts are in [ADR-0031](0031-dsse-ed25519-receipts.md). Audit durability (flushing, fsync) is in [ADR-0030](0030-audit-durability.md).
