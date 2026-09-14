<!-- shiin-doc: kind=adr status=proposed implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR template

> [!NOTE]
> **Architecture decision: proposed.**
> Copy this file to `docs/adr/NNNN-short-title.md`, replace every placeholder, and add the new record to the [ADR index](README.md).

This template follows [MADR 4](https://adr.github.io/madr/).
Keep records short. An ADR explains one decision, the options considered, and the consequences.
Detailed rules belong in the specification the ADR links to.

A new record's title is written as `# ADR-NNNN: Title in sentence case`.
Its metadata comment uses `kind=adr` and one of the statuses
`proposed`, `accepted`, `deferred`, `superseded`, or `deprecated`.

---

- **Date:** YYYY-MM-DD
- **Deciders:** names or handles
- **Supersedes:** ADR-NNNN, or "none"
- **Superseded by:** ADR-NNNN, or "none"
- **Related:** links to specifications, other ADRs, and issues

## Context and problem statement

Describe the situation and the question that needs an answer, in two to five sentences.

## Decision drivers

- A force or constraint that shapes the decision
- Another one

## Considered options

1. Option one
2. Option two
3. Option three

## Decision outcome

Chosen option: "Option one", because it best satisfies the decision drivers in the following way.

### Consequences

- Good, because of a benefit.
- Bad, because of a cost that is accepted.

### Confirmation

How compliance with the decision is checked: a test, a CI check, a review rule, or a spike with exit criteria.

## Pros and cons of the options

### Option one

- Good, because of a strength.
- Bad, because of a weakness.

### Option two

- Good, because of a strength.
- Bad, because of a weakness.

## More information

Links, prior art, and conditions under which the decision should be revisited.
