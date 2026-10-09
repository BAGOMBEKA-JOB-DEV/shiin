<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0040: Platform support tiers

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Platform support](../reference/platform-support.md), [Toolchain and MSRV](../engineering/toolchain-and-msrv.md), [Unsafe code policy](../engineering/unsafe-policy.md), [Roadmap](../project/roadmap.md)

## Context and problem statement

Shiin runs on the developer's machine and must decide how many platforms to officially support with CI, prebuilt binaries, and tests.
Supporting every platform from day one spreads the project thin.
The project needed a tiering scheme that sets expectations for where Shiin works, is tested, and ships binaries.

## Decision drivers

- Tier 1 must be the platforms the maintainers use and CI tests.
- Tier 2 is best-effort with no CI guarantee.
- Tier 3 is community-supported or not yet ported.

## Considered options

1. Three tiers: Tier 1 (tested + binaries), Tier 2 (builds but no CI), Tier 3 (community).
2. Binary support only for the latest two stable releases of each OS.
3. No tiers; support everything.

## Decision outcome

Chosen option: "Three tiers with CI and binaries for Tier 1 only", because it sets clear expectations and keeps CI focused.

- **Tier 1:** Linux (x86-64), macOS (aarch64), Windows (x86-64). CI runs on every pull request. Prebuilt binaries shipped.
- **Tier 2:** Linux (aarch64), macOS (x86-64). Builds in CI but not guaranteed.
- **Tier 3:** All other platforms. No CI guarantee; community builds welcome.
- Performance budgets and receipts are verified on Tier 1 platforms (exit criterion: v0.4).
- Windows promotion from Tier 2 to Tier 1 is planned for v0.6.

### Consequences

- Good, because CI and binaries focus on a manageable set of platforms.
- Good, because tier expectations are clear to users.
- Bad, because some platforms get no prebuilt binaries.

### Confirmation

- The [platform support page](../reference/platform-support.md) lists all tiers.
- CI runs on Tier 1 platforms only.

## Pros and cons of the options

### Three tiers

- Good, because it is simple and sets clear expectations.
- Bad, because Tier 3 users get no guarantees.

### Latest two stable releases only

- Good, because it limits OS version support.
- Bad, because it does not distinguish architecture or CI coverage.

### Support everything

- Good, because it maximizes reach.
- Bad, because testing and binary distribution become intractable.

## More information

The [roadmap](../project/roadmap.md) mentions Windows promotion to Tier 1 in v0.6. The performance budgets page specifies latency and memory targets per tier.
