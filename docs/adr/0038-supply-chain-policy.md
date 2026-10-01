<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0038: Supply chain policy

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Dependencies and supply chain](../engineering/dependencies-and-supply-chain.md), [ADR-0037](0037-unsafe-code-policy.md)

## Decision outcome

Every dependency is admitted with a reason.
Reviewers weigh unsafe usage, testing, soundness history, and safe alternatives.
cargo-vet is enforced in CI from v0.4.

## Consequences
- Dependencies are updated through ordinary pull requests.
- A soundness advisory is treated as a potential vulnerability.
