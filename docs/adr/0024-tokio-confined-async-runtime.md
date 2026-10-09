<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0024: Tokio, confined

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Workspace and crates](../engineering/workspace-and-crates.md), [Async and concurrency](../engineering/async-and-concurrency.md), [ADR-0010](0010-pure-synchronous-engine.md), [ADR-0002](0002-rust-for-product-and-tooling.md)

## Context and problem statement

The daemon (`shiind`, v0.3) and the MCP proxy (v0.5) are long-lived, I/O-bound processes that need asynchronous runtimes.
But the engine ([ADR-0010](0010-pure-synchronous-engine.md)) is pure and synchronous.
The project needed an async runtime that is confined to the parts that need it, without leaking into the engine.

## Decision drivers

- Only the daemon and MCP proxy need async.
- The engine must remain synchronous and panic-safe.
- Contributors should use one well-supported async runtime.

## Considered options

1. Tokio, used only in binaries and the IPC crate, never in the engine crates.
2. async-std as a lighter alternative.
3. No async runtime; use blocking I/O everywhere.

## Decision outcome

Chosen option: "Tokio, confined to binaries and IPC", because Tokio is the most mature Rust async runtime and integrates with the crates the project already considers.

- Tokio is a dependency of `shiind` and `shiin-ipc` only.
- The engine crates (`shiin-schema`, `shiin-classify`, `shiin-policy`, `shiin-engine`) depend on no async runtime.
- The MCP proxy uses Tokio through its MCP framework dependency.
- A planned `xtask` check inspects the non-dev dependency tree of engine crates and fails if a forbidden crate appears.

### Consequences

- Good, because it gives the daemon and proxy good async I/O performance.
- Good, because the engine stays pure and synchronous.
- Bad, because Tokio adds compile-time and binary-size weight to the daemon and proxy.

### Confirmation

- [Workspace and crates](../engineering/workspace-and-crates.md) forbids networking and async in the engine crates.
- CI runs a planned `xtask` check for the dependency-tree rule.

## Pros and cons of the options

### Tokio, confined

- Good, because it is mature and well-supported.
- Bad, because compile times are longer for async crates.

### async-std

- Good, because it is lighter.
- Bad, because it has smaller community and fewer integrations.

### No async runtime

- Good, because it avoids runtime dependency.
- Bad, because the daemon needs to handle many concurrent connections efficiently.

## More information

The [async and concurrency engineering page](../engineering/async-and-concurrency.md) describes the confinement rules. The wasm32 build of the engine is a purity guard against runtime leakage.
