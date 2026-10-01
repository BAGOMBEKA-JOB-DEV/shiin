<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0033: Enforcement levels

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Enforcement levels](../security/enforcement-levels.md), [Adapter contract](../spec/adapter-contract.md)

## Decision outcome

Every adapter declares an enforcement level: L1 cooperative, L2 mediated, or L3 OS-enforced.
Each adapter's documentation states its enforcement level and what it cannot see.

## Consequences
- Protection is never overstated.
- L3 is a future enforcement level.
