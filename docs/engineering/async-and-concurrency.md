<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Async and concurrency

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's async and concurrency rules ([ADR-0024](../adr/0024-tokio-confined-async-runtime.md)).

## Engine

The engine is pure and synchronous.
It performs no I/O and uses no async runtime.

## Adapters and services

Adapters and services may use Tokio, but it is confined:

- The runtime is created in the binary and shut down before exit.
- Libraries do not create their own runtime.
- Engine crates never depend on an async runtime.

## Concurrency

Evaluation is deterministic.
Concurrent evaluations must not share mutable state.

## Read next
- [Workspace and crates](workspace-and-crates.md)
- [Toolchain and MSRV](toolchain-and-msrv.md)