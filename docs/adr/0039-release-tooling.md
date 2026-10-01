<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0039: Release tooling

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Release engineering](../engineering/release-engineering.md), [Verifying releases](../security/verifying-releases.md)

## Decision outcome

Commits follow Conventional Commits and carry DCO sign-offs.
Release tooling generates changelogs from them.
Signed release binaries with a software bill of materials are produced for every release.

## Consequences
- Changelogs are generated, not written by hand.
- Binaries are self-contained and fully static on musl targets.
