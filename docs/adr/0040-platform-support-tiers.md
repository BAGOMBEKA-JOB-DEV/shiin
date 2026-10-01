<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0040: Platform support tiers

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Platform support](../reference/platform-support.md)

## Decision outcome

Tier 1 platforms are Linux x86_64 and macOS 13+ (x86_64 and arm64), tested in CI.
Tier 2 is Windows 10+ (x86_64), which builds and is promoted toward Tier 1 in v0.6.
Tier 3 is other architectures, not guaranteed to build.

## Consequences
- Tier 1 platforms meet the performance budgets.
- Tier 2 platforms are documented and tracked.
