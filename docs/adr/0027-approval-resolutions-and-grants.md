<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0027: Approval resolutions and grants

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Approval protocol](../spec/approval-protocol.md), [Grant](../glossary.md#grant)

## Decision outcome

An approver resolves an approval request with `allow_once`, `allow_always`, or `deny`.
`allow_once` creates a single-use grant bound to the digest of one action request.
`allow_always` creates a scoped grant covering an agent, action types, a resource pattern, and a workspace, which expires.
A grant never overrides `deny`.

## Consequences
- Grants are stored and auditable.
- Self-approval through an intercepted channel is impossible by test.
