<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0010: Pure synchronous engine

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Engine](../glossary.md#engine), [ADR-0009](0009-deterministic-enforcement.md), [Workspace and crates](../engineering/workspace-and-crates.md)

## Decision outcome

The engine is pure and synchronous.
It performs no I/O and depends on no async runtime.
It returns a trace as part of its result, which keeps it pure and makes explanations reproducible.

## Consequences
- The engine builds for WebAssembly as a purity guard from v0.4.
- Engine crates never open network connections.