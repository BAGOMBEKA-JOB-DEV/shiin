# Governance

This document describes how decisions are made in Shiin.
It follows [ADR-0005](docs/adr/0005-governance-model.md).

## Roles

| Role | Who | What they can do |
|---|---|---|
| Contributor | Anyone who opens an issue or pull request | Propose changes, review, discuss |
| Maintainer | Listed in [MAINTAINERS.md](MAINTAINERS.md) | Review and merge, accept decision records, cut releases, handle security reports |
| Founding maintainer | The project's originator | Everything a maintainer can do, and acts as tiebreaker until a maintainer council exists |

## How decisions are made

The project works by **lazy consensus**: a proposal that draws no objection within a reasonable review period is accepted.
Anyone may object, and an objection is resolved by discussion rather than by vote.
If consensus cannot be reached, the founding maintainer decides and records the reasoning.

Decisions are recorded, not remembered:

- Significant technical choices become [architecture decision records](docs/adr/README.md).
- Changes to accepted specifications, governance, or stable surfaces go through the [RFC process](docs/rfcs/README.md).
- Security-sensitive decisions, including vulnerability handling and releases, need maintainer approval.

## Becoming a maintainer

There is no fixed quota of contributions.
A contributor is invited to become a maintainer when they have shown sustained, high-quality participation, good judgment about scope and risk, and care for the project's honesty rules.
Existing maintainers decide by consensus, and the change is recorded in [MAINTAINERS.md](MAINTAINERS.md).

A maintainer who has been inactive for six months may be moved to emeritus status, with no implied criticism, and may return on request.

## Forming a maintainer council

The current single-maintainer model is explicitly temporary.
A council is formed when either of these becomes true:

- there are three or more active maintainers, or
- contributors from two or more organizations are maintaining the project regularly.

The council then takes over tiebreaking, adding and removing maintainers, and approving RFCs, and this document is updated by RFC to describe it.

## Changing this document

Changes to governance require an RFC and the agreement of the maintainers.
