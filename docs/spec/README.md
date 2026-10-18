<!-- shiin-doc: kind=index status=draft implementation=partial milestone=m0 reviewed=2026-10-18 -->

# About the specifications

> [!NOTE]
> **Project status: pre-alpha.**
> Core contracts have source implementations and conformance tests; approval, evidence, and local API implementations are partial and still need milestone validation.
> The first experimental CLI and Unix-daemon release is available; contracts remain subject to change before 1.0.
> Tests check implemented wire values against the schemas.

The specifications define the contracts that make Shiin interchangeable: the [action request](action-request.md), the [decision](decision.md), the [evaluation](evaluation.md) pipeline, and the [approval protocol](approval-protocol.md), along with JSON Schemas and test vectors that other tools can implement.

Each page distinguishes implemented behavior from remaining design work.
The [roadmap](../project/roadmap.md) lists the milestone that delivers each specification's implementation.

## Status and lifecycle

| Status | Meaning |
|---|---|
| `draft` | The specification is under active revision. Field names, IDs, and schemas may change. |
| `proposed` | The specification has stabilized enough to send to implementers for review. |
| `accepted` | The content is settled and may be implemented against. Before 1.0 a correction may still raise the draft minor version, recorded in the revision history; after 1.0 a change requires a superseding version. |
| `superseded` | A newer version replaces this one. Links to the replacement. |

The `status` field in each page's metadata reflects where the specification is in this lifecycle, not whether code exists.

## Versioning

Each specification states its version near the top, in the form `shiin.<name>/v<major>-draft.<minor>`:

- The major version is part of the schema `$id` URN (for example, `urn:shiin:schema:action-request:v1`).
- A draft minor number (`.1`, `.2`, ...) increments for any change that is not a new major version.
- A new major version is issued when a change is not backward compatible.
- The version string in the specification text, the `$id` in the schema, and the schema file name must all agree.

## Requirement IDs

Every normative requirement in a specification starts with a bold, bracketed requirement ID, for example `**[AR-001]**`.

- IDs are unique across all documentation and are defined only in their owning `kind=spec` page.
- The prefix indicates which specification owns the requirement:

| Prefix | Specification |
|---|---|
| `AT` | [Action types](action-types.md) |
| `AR` | [Action request](action-request.md) |
| `DEC` | [Decision](decision.md) |
| `EVAL` | [Evaluation](evaluation.md) |
| `ID` | [Identity](identity.md) |
| `POL` | [Policy model](policy-model.md) |
| `SYN` | [Policy syntax](policy-syntax.md) |
| `INT` | [Intent binding](intent-binding.md) |
| `RISK` | [Risk signals](risk-signals.md) |
| `APR` | [Approval protocol](approval-protocol.md) |
| `AUD` | [Audit events](audit-events.md) |
| `RCPT` | [Receipts](receipts.md) |
| `ADP` | [Adapter contract](adapter-contract.md) |
| `API` | [Local API](local-api.md) |
| `SEC` | [Security model](../security/security-model.md) |

## Schemas and test vectors

- JSON Schemas live in [`schemas/`](schemas/) and are named `<name>.v<major>.schema.json`.
- Shared definitions live in `common.v1.schema.json` and are referenced by `$id`.
- Test vectors live in [`test-vectors/`](test-vectors/) and are validated against `test-vector.v1.schema.json`.

## Specification list

| Specification | Milestone | Schema |
|---|---|---|
| [Action types](action-types.md) | v0.1 | — |
| [Action request](action-request.md) | v0.1 | [schema](schemas/action-request.v1.schema.json) |
| [Decision](decision.md) | v0.1 | [schema](schemas/decision.v1.schema.json) |
| [Evaluation](evaluation.md) | v0.1 | [schema](schemas/evaluation.v1.schema.json) |
| [Identity](identity.md) | v0.1 | — |
| [Policy model](policy-model.md) | v0.1 | — |
| [Policy syntax](policy-syntax.md) | v0.1 | [schema](schemas/policy-syntax.v1.schema.json) |
| [Intent binding](intent-binding.md) | v0.2 | [schema](schemas/intent-binding.v1.schema.json) |
| [Risk signals](risk-signals.md) | v0.1 | [schema](schemas/risk-signals.v1.schema.json) |
| [Approval protocol](approval-protocol.md) | v0.3 | [schema](schemas/approval-protocol.v1.schema.json) |
| [Audit events](audit-events.md) | v0.1 | [schema](schemas/audit-events.v1.schema.json) |
| [Receipts](receipts.md) | v0.4 | [receipt](schemas/receipts.v1.schema.json), [checkpoint](schemas/checkpoint.v1.schema.json), [encrypted replay](schemas/encrypted-replay.v1.schema.json) |
| [Adapter contract](adapter-contract.md) | v0.1 | — |
| [Local API](local-api.md) | v0.5 | [schema](schemas/local-api.v1.schema.json) |
| [Test vectors](test-vectors/README.md) | m0 | [schema](schemas/test-vector.v1.schema.json) |

## Reading the specifications

1. Start with [How Shiin works](../concepts/how-shiin-works.md) for the end-to-end flow.
2. Read the [action request](action-request.md) and [decision](decision.md) for the core messages.
3. Read [evaluation](evaluation.md) for how decisions are combined.
4. Read the [adapter contract](adapter-contract.md) for integration.
5. Consult the [test vectors](test-vectors/README.md) for concrete examples.

## Related reading

- [How Shiin works](../concepts/how-shiin-works.md)
- [Use cases](../overview/use-cases.md)
- [Glossary](../glossary.md)
- [JSON Schemas](schemas/)
- [Test vectors](test-vectors/README.md)
