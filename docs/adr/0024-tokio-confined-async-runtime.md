<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0024: Tokio, confined

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Local API](../spec/local-api.md), [Async and concurrency](../engineering/async-and-concurrency.md), [ADR-0010](0010-pure-synchronous-engine.md)

## Decision outcome

Adapters and services may use Tokio, but it is confined.
The runtime is created in the binary and shut down before exit.
Libraries do not create their own runtime.
Engine crates never depend on an async runtime.

## Consequences
- The engine stays pure and synchronous.
- The daemon can use async I/O.
