<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0017: JSON wire format and schemas

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Action request](../spec/action-request.md), [Decision](../spec/decision.md), [JSON Schemas](../project/docs-style-guide.md#json-schemas)

## Decision outcome

The wire format is JSON.
Schemas live in `docs/spec/schemas/` and are named `<name>.v<major>.schema.json`.
Every schema declares Draft 2020-12 and an `$id` of the form `urn:shiin:schema:<name>:v<major>`.

## Consequences
- Schemas are validated by `cargo xtask docs-check`.
- Test vectors are validated against the test-vector schema.
