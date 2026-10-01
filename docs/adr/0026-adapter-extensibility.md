<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0026: Adapter extensibility

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Adapter contract](../spec/adapter-contract.md), [ADR-0025](0025-adapter-roadmap.md)

## Decision outcome

Adapters are not dynamically extensible through plugins.
New adapters are added through the RFC process and an ADR.
Custom agents integrate through the local API.

## Consequences
- Adapters are reviewed and tested like product code.
- Third parties cannot ship plugins.
