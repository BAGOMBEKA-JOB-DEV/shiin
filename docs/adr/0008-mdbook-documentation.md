<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0008: mdBook documentation

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Documentation tooling and CI](../project/docs-tooling-and-ci.md), [ADR-0002](0002-rust-for-product-and-tooling.md)

## Decision outcome

The documentation builds as a book with mdBook 0.5.4.
Every documentation tool is installed with `cargo install --locked`.
All checks that run in CI can be run locally with the same command.

## Consequences
- The built site is written to `target/book/`.
- `mdbook serve docs` previews it with live reload.