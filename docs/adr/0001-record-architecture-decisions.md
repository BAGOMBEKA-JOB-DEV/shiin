<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0001: Record architecture decisions

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [ADR template](template.md), [RFC process](../rfcs/README.md), [Documentation style guide](../project/docs-style-guide.md)

## Context and problem statement

Shiin is being designed in full before any product code is written, so many consequential choices are made at the same time.
Contributors who arrive later need to know what was decided, why, and which alternatives were rejected.
Without that record, settled questions get reopened, or a decision is reversed by someone who never saw its reasons.
Discussion in issues and pull requests is hard to find later and carries no status.
The project needs a durable, reviewable, and findable record of each significant decision.

## Decision drivers

- Decisions are findable next to the specifications they shape, and versioned with them.
- A reader can tell whether a decision is in effect, awaiting confirmation, or replaced.
- History is preserved: a reversed decision still explains why it was once made.
- Records are plain Markdown that renders on GitHub and in the documentation book ([ADR-0008](0008-mdbook-documentation.md)).
- Writing a record is cheap enough that people actually do it.

## Considered options

1. MADR 4 records in `docs/adr/`, superseded but never deleted
2. A shorter context, decision, and consequences format in `docs/adr/`
3. Decisions recorded only in issues, pull requests, and discussions
4. A wiki of decisions outside the repository

## Decision outcome

Chosen option: "MADR 4 records in `docs/adr/`", because it keeps decisions versioned with the specifications
and makes every record state the options considered and how the decision will be confirmed.

- Each record follows the [ADR template](template.md), which is based on [MADR 4](https://adr.github.io/madr/).
- Records are numbered sequentially as `NNNN-short-title.md`, and a number is never reused.
- Every record is listed with its status in the [ADR index](README.md).
- Records are never deleted.
  A decision that changes is superseded by a new record, and the two records link to each other.
- Changes to accepted specifications, new action-type namespaces, and policy-language features go through the [RFC process](../rfcs/README.md), and an accepted RFC produces an ADR.
- A record whose confirmation step has not yet run stays `proposed`, even when its recommendation is firm.
- A record explains a decision.
  Normative rules live in the specification the record links to.

### Consequences

- Good, because the reasons for a design outlive the conversations in which it was made.
- Good, because the `proposed` status keeps unconfirmed decisions visible, such as [ADR-0014](0014-cedar-policy-engine-with-toml-front-end.md).
- Good, because superseding instead of rewriting keeps an honest history.
- Bad, because significant changes carry the extra work of writing a record.
- Bad, because superseded records accumulate, and readers must follow links to reach the current decision.

### Confirmation

- `cargo xtask docs-check` checks that ADR numbers are unique, that every title starts with `# ADR-NNNN:`, that the index links every record, and that every `ADR-NNNN` reference points to an existing record.
- Review rule: a pull request that changes what an accepted record decides is not merged.
  It adds a superseding record instead.
  Fixes to wording, links, and status fields are allowed.
- Review rule: a pull request that meets a criterion in [When to write an ADR](README.md#when-to-write-an-adr) is not merged without a linked record.

## Pros and cons of the options

### MADR 4 records in the repository

- Good, because records are reviewed in the same pull requests as the specifications they affect.
- Good, because the template's considered-options and confirmation sections force the reasoning to be written down.
- Bad, because the template is longer than a minimal format.

### A shorter format in the repository

- Good, because records are quicker to write.
- Bad, because rejected options and confirmation steps are easy to leave out, and those are what later readers most need.

### Issues, pull requests, and discussions only

- Good, because no extra files or process are needed.
- Bad, because decisions are scattered across threads, have no status, and cannot be checked automatically.

### A wiki outside the repository

- Good, because pages are easy to edit.
- Bad, because wiki pages are not versioned with the specifications and do not ship with the documentation.

## More information

- Status meanings and the process for adding a record are in the [ADR index](README.md).
- Revisit this decision if the number of records makes the index hard to navigate.
