<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0004: Apache-2.0 license and DCO

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [NOTICE](../../NOTICE), [LICENSE](../../LICENSE)

## Context and problem statement

The project needed a license and a contribution process.
The license had to be permissive, widely understood, and compatible with the dependencies Shiin will use.
The contribution process had to record authorship for copyright purposes.

## Considered options

1. Apache-2.0 with DCO sign-off
2. MIT with CLA
3. GPL-3.0

## Decision outcome

Chosen option: Apache-2.0 with DCO sign-off.

- Apache-2.0 is permissive and compatible with the Rust ecosystem.
- Every commit carries a Developer Certificate of Origin sign-off, which `git commit -s` adds.
- No contributor license agreement is required.

## Consequences

- Contributions are accepted under the same license.
- Commits need a DCO sign-off.
- See [NOTICE](../../NOTICE) for attribution.

## Confirmation

- The license file is in the repository root.
- CI checks for sign-offs.