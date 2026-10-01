<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0034: Edition, MSRV, and toolchain

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Toolchain and MSRV](../engineering/toolchain-and-msrv.md), [ADR-0002](0002-rust-for-product-and-tooling.md)

## Decision outcome

The edition is 2024.
The pinned toolchain is 1.98.1.
The MSRV is 1.96 before 1.0, and latest stable minus 6 minor versions after 1.0 for library crates.

## Consequences
- Every contributor uses the pinned toolchain.
- The MSRV is checked with cargo-hack.
