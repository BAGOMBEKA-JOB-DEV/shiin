<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0025: Adapter roadmap

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Adapter contract](../spec/adapter-contract.md), [Adapters](../concepts/adapters.md)

## Decision outcome

Adapters are built for hosts in milestone order: Claude Code (v0.1), Codex CLI and Cursor (v0.2), MCP proxy (v0.5).
Each adapter declares its enforcement level and what it cannot see.

## Consequences
- Adapters are added through pull requests with an ADR.
- Dynamic-library plugins for adapters are not planned.
