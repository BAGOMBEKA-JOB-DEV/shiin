<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0005: Governance model

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [ADR-0001](0001-record-architecture-decisions.md), [ADR-0006](0006-vulnerability-reporting.md), [Governance overview](../../GOVERNANCE.md)

## Context and problem statement

As an open-source project gains contributors, it needs a clear model for how decisions are made and how disagreements are resolved.
Shiin needed a governance structure that scales from the founding maintainer to a community without requiring a corporate sponsor.

## Decision drivers

- Simplicity for an early-stage project.
- Clear authority for security and release decisions.
- A path to broader contribution as the project grows.
- Alignment with the DCO-only contribution model.

## Considered options

1. Benevolent dictator for life (BDFL).
2. Lazy consensus with a maintainer group.
3. A formal steering committee from day one.

## Decision outcome

Chosen option: "Lazy consensus with a maintainer group", because it is lightweight for a small project and grows naturally into a broader governance model.

- The project works by lazy consensus: proposals that receive no objection after a reasonable review period are considered accepted.
- The founding maintainer is the initial sole maintainer, listed in `MAINTAINERS.md`.
- Security-sensitive decisions (vulnerability handling, releases) require maintainer approval.
- As the project grows, maintainers are added by consensus of existing maintainers.
- This model is documented in `GOVERNANCE.md`.

### Consequences

- Good, because it minimizes process overhead while the contributor base is small.
- Good, because it has a clear escalation path if consensus fails.
- Bad, because the founding maintainer is a single point of failure until more maintainers are added.

### Confirmation

- `GOVERNANCE.md` and `MAINTAINERS.md` exist at the repository root and describe the model.
- ADRs are accepted and superseded by maintainers, as described in [ADR-0001](0001-record-architecture-decisions.md).

## Pros and cons of the options

### Benevolent dictator for life

- Good, because decisions are fast.
- Bad, because it centralizes power and has no clear succession plan.

### Lazy consensus with a maintainer group

- Good, because it is lightweight and scales.
- Bad, because it can slow down when maintainers disagree.

### Formal steering committee from day one

- Good, because it distributes power broadly.
- Bad, because it is heavyweight for a project with no code shipped yet.

## More information

The governance model is revisited when the maintainer group grows beyond three members. See [ADR-0005](0005-governance-model.md) for the full record.
