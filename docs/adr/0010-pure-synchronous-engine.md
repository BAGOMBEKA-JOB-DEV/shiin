<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0010: Pure synchronous engine

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Glossary](../glossary.md#engine), [Evaluation](../spec/evaluation.md), [ADR-0009](0009-deterministic-enforcement.md), [Workspace and crates](../engineering/workspace-and-crates.md)

## Context and problem statement

The engine evaluates action requests against policy.
If the engine performs I/O, opens network connections, or uses asynchronous runtimes, it becomes harder to test, harder to reason about, and harder to embed in different hosts.
The project needed to decide what the engine is allowed to do.

## Decision drivers

- Replay and verification require the engine to be a pure function.
- CI builds `shiin-engine` for `wasm32-unknown-unknown` as a purity guard.
- The engine must be fast and predictable on the hook path.

## Considered options

1. A pure, synchronous engine that performs no I/O and depends on no async runtime.
2. An async engine with I/O for loading policy and grants on demand.
3. A hybrid engine that is synchronous for policy rules but async for lookups.

## Decision outcome

Chosen option: "A pure, synchronous engine", because purity is what makes determinism and replay possible, and it keeps the dependency graph small.

- `shiin-engine` performs no I/O of any kind.
- It takes an action request and a policy snapshot and returns a decision.
- It depends on `shiin-policy` and `shiin-schema` only.
- It never depends on `shiin-classify` (classification happens in the adapter before the engine is called).
- From v0.4, CI builds `shiin-engine` for `wasm32-unknown-unknown` to catch I/O dependencies.

### Consequences

- Good, because the engine is fully testable with deterministic unit tests.
- Good, because the engine can be embedded in any runtime, including WebAssembly.
- Bad, because the engine cannot fetch remote policy or grants; that is the adapter's job, which provides the snapshot.

### Confirmation

- `shiin-classify` and `shiin-policy` may read the local file system where their job requires it.
- [Workspace and crates](../engineering/workspace-and-crates.md) forbids networking in the engine crates.
- CI builds for `wasm32-unknown-unknown` (planned: v0.4).

## Pros and cons of the options

### Pure, synchronous engine

- Good, because it is deterministic, testable, and embeddable.
- Bad, because it cannot do on-demand lookups.

### Async engine with I/O

- Good, because it can fetch policy and data on demand.
- Bad, because I/O breaks determinism and complicates testing.

### Hybrid engine

- Good, because it balances purity with flexibility.
- Bad, because the boundary between synchronous rules and async lookups is a source of bugs.

## More information

The [decision pipeline](../architecture/decision-pipeline.md) describes what data the snapshot contains. The adapter contract specifies how the adapter assembles the snapshot from policy layers, grants, and tasks.
