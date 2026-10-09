<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0027: Approval resolutions and grants

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Approval protocol](../spec/approval-protocol.md), [Glossary](../glossary.md#grant), [Glossary](../glossary.md#resolution), [Use case UC-05](../overview/use-cases.md#uc-05-approve-consequential-operations), [ADR-0028](0028-deny-with-ticket-for-non-waiting-hosts.md)

## Context and problem statement

When a decision is `pending`, a human must resolve it.
The human's answer is a resolution, which may produce a stored grant that satisfies future similar requests.
The project needed to define what resolutions are possible and how grants are scoped, so that an approver's intent is captured precisely and cannot be abused.

## Decision drivers

- Approvals must be precise: one-time, scoped, or denied.
- Grants must never override a `deny`.
- The approval format must be versioned and implementable by any channel.

## Considered options

1. Three resolutions: `allow_once`, `allow_always`, `deny`, with single-use and scoped grants.
2. A free-form approval with arbitrary parameters.
3. Binary only: allow or deny, no stored grants.

## Decision outcome

Chosen option: "Three resolutions with single-use and scoped grants", because it gives approvers the precision they need without unbounded complexity.

- A resolution is `allow_once`, `allow_always`, or `deny`.
- An `allow_once` resolution creates a single-use grant bound to the digest of one action request.
- An `allow_always` resolution creates a scoped grant covering an agent, action types, a resource pattern, and a workspace, with an expiry.
- A grant can satisfy only a grantable `pending` result. It never overrides `deny`.
- Approvers approve; the resulting decision is `allow` — never the reverse.

### Consequences

- Good, because approvers can choose the minimum scope they intend.
- Good, because grants are typed and bounded in time and scope.
- Bad, because scoped grants add complexity to grant management and revocation.

### Confirmation

- The [approval protocol specification](../spec/approval-protocol.md) defines the message formats and state machine.
- A property test suite verifies the approval state machine (exit criterion: v0.3).

## Pros and cons of the options

### Three resolutions, single-use and scoped grants

- Good, because it is precise and auditable.
- Bad, because it is more complex than binary.

### Free-form parameters

- Good, because it is flexible.
- Bad, because it is hard to audit and easy to over-grant.

### Binary, no grants

- Good, because it is simple.
- Bad, because it forces the human to approve every action, causing fatigue.

## More information

The glossary defines [grant](../glossary.md#grant) and [resolution](../glossary.md#resolution). Deny-with-ticket for non-waiting hosts is in [ADR-0028](0028-deny-with-ticket-for-non-waiting-hosts.md).
