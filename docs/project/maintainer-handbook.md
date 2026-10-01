<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Maintainer handbook

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's maintainer rules.

## Responsibilities

Maintainers:

- Review and merge pull requests.
- Accept or reject RFCs.
- Cut releases.
- Respond to vulnerability reports.

## Merging

A pull request merges when:

- CI passes.
- It has a DCO sign-off.
- It has at least one maintainer review.
- A change to an accepted ADR adds a superseding ADR.

## Releases

A release is cut from `main` when the milestone's exit criteria are met.

## Branch protection

A failing check blocks merging once branch protection is configured.

## Read next

- [Governance model](../adr/0005-governance-model.md)
- [Release engineering](../engineering/release-engineering.md)