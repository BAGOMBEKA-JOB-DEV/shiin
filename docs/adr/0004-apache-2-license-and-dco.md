<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0004: Apache-2.0 license and DCO

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Documentation style guide](../project/docs-style-guide.md), [README](../../README.md)

## Context and problem statement

Shiin is open-source software intended for adoption by developers and teams.
It needed a license that is permissive enough for commercial and personal use, includes a patent grant, and is well understood by the ecosystem.
Contributions needed a lightweight sign-off mechanism, not a separate contributor license agreement that creates friction for first-time contributors.

## Decision drivers

- Permissive enough for broad adoption, including by agent vendors.
- Patent protection for contributors and users.
- Minimal contribution overhead.
- Compatibility with the Cedar dependency's license.

## Considered options

1. Apache License 2.0 with Developer Certificate of Origin.
2. MIT License.
3. BSD 3-clause.
4. A contributor license agreement in addition to the license.

## Decision outcome

Chosen option: "Apache License 2.0 with Developer Certificate of Origin", because Apache 2.0 provides an explicit patent grant that MIT and BSD do not, and the DCO adds no contributor friction.

- The license is Apache-2.0, as set in `[workspace.package].license` in `Cargo.toml`.
- Every commit requires a DCO sign-off (`git commit -s`); no contributor license agreement is used.
- This choice is recorded in [ADR-0004](0004-apache-2-license-and-dco.md) and referenced from [CONTRIBUTING.md](../../CONTRIBUTING.md).

### Consequences

- Good, because contributors and users get a well-understood license with patent protection.
- Good, because the DCO sign-off is a one-line commit flag, not a separate document to sign.
- Bad, because Apache 2.0 is slightly longer than MIT or BSD, though this does not affect usage.

### Confirmation

- The `LICENSE` file and `NOTICE` file are present at the repository root.
- All crates inherit `license = "Apache-2.0"` from the workspace.

## Pros and cons of the options

### Apache 2.0 with DCO

- Good, because of the explicit patent grant.
- Good, because DCO is lightweight.
- Bad, because it is longer than MIT.

### MIT

- Good, because it is short and widely recognized.
- Bad, because it has no explicit patent grant.

### BSD 3-clause

- Good, because it is short and permissive.
- Bad, because it lacks a patent grant and adds a clause that rarely matters in practice.

### Contributor license agreement

- Good, because it gives the project legal flexibility.
- Bad, because it creates friction for contributors and conflicts with the goal of minimal overhead.

## More information

The DCO is documented at [developercertificate.org](https://developercertificate.org/). The Apache 2.0 license text is in `LICENSE`. Attribution notices are in `NOTICE`.
