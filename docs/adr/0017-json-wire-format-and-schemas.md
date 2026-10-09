<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0017: JSON wire format and schemas

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Evaluation](../spec/evaluation.md), [JSON Schemas](../spec/README.md), [ADR-0002](0002-rust-for-product-and-tooling.md), [ADR-0023](0023-json-rpc-local-api.md), [ADR-0042](0042-language-bindings.md)

## Context and problem statement

Shiin communicates with adapters, agents, and other tools.
The wire format must be language-neutral, versioned, and validated so that integrators in other languages can interoperate.
Schemas are needed so that implementations can be tested against the specification.

## Decision drivers

- Must be language-neutral.
- Must be versioned so specifications can evolve.
- Must be independently implementable without a Rust dependency.
- Must be machine-validated so conformance can be tested.

## Considered options

1. JSON with JSON Schema, identified by URN identifiers.
2. Protocol Buffers with generated code.
3. Cap'n Proto with a schema compiler.

## Decision outcome

Chosen option: "JSON with JSON Schema, identified by URN identifiers", because it is the most interoperable and requires no code generation.

- Action requests, decisions, approval messages, and audit events are serialized as JSON.
- Each message type has a JSON Schema under `docs/spec/schemas/`.
- Schemas are identified by URNs of the form `urn:shiin:schema:<name>:v<major>`.
- URN identifiers keep schema identity independent of any web domain.
- Shared definitions live in `common.v1.schema.json` and are referenced by `$id`.

### Consequences

- Good, because any language with a JSON parser can interoperate.
- Good, because schemas enable conformance testing.
- Bad, because JSON is more verbose than binary formats, but Shiin's messages are small.

### Confirmation

- `cargo xtask docs-check` compiles and validates all schemas.
- Every JSON example in specifications carries a validation marker and is checked against its schema.

## Pros and cons of the options

### JSON with JSON Schema

- Good, because it is universally supported.
- Good, because schemas are testable.
- Bad, because it is verbose compared to binary.

### Protocol Buffers

- Good, because it is compact and has code generation for many languages.
- Bad, because it requires a code generator and is not human-readable.

### Cap'n Proto

- Good, because it supports zero-copy reads.
- Bad, because it has limited language support and is not human-readable.

## More information

The [specification README](../spec/README.md) lists all schema files. Test vectors encoding the golden use cases are in `docs/spec/test-vectors/`.
