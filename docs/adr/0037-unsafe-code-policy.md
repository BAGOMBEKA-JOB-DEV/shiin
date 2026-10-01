<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0037: Unsafe code policy

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Unsafe code policy](../engineering/unsafe-policy.md), [ADR-0002](0002-rust-for-product-and-tooling.md)

## Decision outcome

Unsafe code is forbidden everywhere except `shiin-platform`, which is created only when a concrete feature needs an operating-system facility that no safe crate provides.
Every unsafe block carries a `// SAFETY:` comment.

## Consequences
- `unsafe_code = forbid` in the workspace lints.
- `shiin-platform` declares its own lint table.
