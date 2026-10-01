<!-- shiin-doc: kind=rfc status=accepted implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# RFC process

> [!NOTE]
> **RFC: accepted.**
> This page defines the process for proposing changes to Shiin's specifications.

## When to use an RFC

An RFC is required when a change:

- Changes an accepted specification.
- Adds a new action-type namespace.
- Adds a policy-language feature.
- Adds, removes, or reorders milestone scope.

## Process

1. Copy the [RFC template](0000-template.md) to `NNNN-short-title.md`.
2. Open a pull request with the proposal.
3. Reviewers discuss and the maintainers accept or reject.
4. An accepted RFC produces an ADR.

## Statuses

| Status | Meaning |
|---|---|
| `draft` | The proposal is written and open for review. |
| `accepted` | The proposal is accepted and produces an ADR. |
| `rejected` | The proposal is not accepted. |
| `superseded` | Replaced by a later RFC. |

## Read next
- [Architecture decision records](../adr/README.md)
- [RFC template](0000-template.md)