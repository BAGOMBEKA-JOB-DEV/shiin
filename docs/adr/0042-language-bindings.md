<!-- shiin-doc: kind=adr status=deferred implementation=n/a milestone=post-1.0 reviewed=2026-09-14 -->

# ADR-0042: Language bindings

> [!NOTE]
> **Architecture decision: deferred.**
> Deferred until after 1.0.

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR-0002](0002-rust-for-product-and-tooling.md), [Local API](../spec/local-api.md)

## Decision outcome

Language bindings are deferred until after 1.0.
Agent builders in other languages integrate through the JSON wire format and, later, bindings.

## Consequences
- The Rust client library is planned for v0.5.
- Other languages wait until the specifications are frozen.
