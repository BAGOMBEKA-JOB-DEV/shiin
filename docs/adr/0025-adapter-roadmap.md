<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0025: Adapter roadmap

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Adapter contract](../spec/adapter-contract.md), [Glossary](../glossary.md#adapter), [Glossary](../glossary.md#enforcement-level), [Landscape](../overview/landscape.md), [Roadmap](../project/roadmap.md)

## Context and problem statement

Shiin needs adapters for each AI coding agent host.
Each host has a different interception mechanism (hooks) and different response protocols.
The project needed a roadmap that covers the most-used hosts first, while keeping the adapter contract general enough that anyone can write a new adapter.

## Decision drivers

- The first adapter must cover the most-used agent today.
- The contract must be host-agnostic so future adapters are straightforward.
- Adapter coverage must be documented with enforcement levels and blind spots.

## Considered options

1. Start with Claude Code (the most popular host), then add Codex CLI and Cursor.
2. Support all hosts simultaneously with a generic adapter.
3. Start with a single open protocol and ignore host-specific hooks.

## Decision outcome

Chosen option: "Claude Code first, then Codex and Cursor, then MCP", because Claude Code has the most adoption and the clearest hook documentation at the time of writing.

- v0.1 delivers the Claude Code hook adapter, running daemonless.
- v0.2 adds Codex CLI and Cursor hook adapters.
- v0.6 may add a shell wrapper for hosts without hooks.
- Every adapter states its enforcement level (L1/L2/L3) and what it cannot see.
- The contract is specified in the [adapter contract](../spec/adapter-contract.md), not in host-specific documentation.

### Consequences

- Good, because the first adapter covers the largest user base.
- Good, because the contract is general and third parties can write adapters.
- Bad, because users of other hosts must wait for their adapter.

### Confirmation

- The [enforcement levels page](../security/enforcement-levels.md) documents what L1 cooperative means for hook adapters.
- Each host's hook behavior is verified and documented on its integration page under `guides/integrations/`.

## Pros and cons of the options

### Claude Code first, then others

- Good, because it covers the largest immediate user base.
- Bad, because it delays support for other hosts.

### All hosts simultaneously

- Good, because it is comprehensive.
- Bad, because hook interfaces differ enough that parallel implementation is error-prone.

### Single open protocol only

- Good, because it is simplest.
- Bad, because most agents do not support a single open protocol; hooks are host-specific.

## More information

The [landscape](../overview/landscape.md) documents each host's hook capabilities with verification dates. See [ADR-0026](0026-adapter-extensibility.md) for how adapters are loaded.
