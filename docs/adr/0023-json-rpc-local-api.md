<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0023: JSON-RPC local API

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Local API](../spec/local-api.md), [ADR-0024](0024-tokio-confined-async-runtime.md)

## Decision outcome

The local API is JSON-RPC 2.0 over a Unix domain socket on Linux and macOS, and a Windows named pipe on Windows.
The transport is local-only.

## Consequences
- Custom agents can ask Shiin for decisions.
- The daemon is unreachable produces a fail-closed `deny`.
