<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0028: Deny-with-ticket for hosts that cannot wait

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Approval protocol](../spec/approval-protocol.md), [Glossary](../glossary.md#deny-with-ticket), [Glossary](../glossary.md#hook), [ADR-0027](0027-approval-resolutions-and-grants.md)

## Context and problem statement

Some host hooks have strict timeouts and cannot wait for a human to approve a `pending` decision.
If the adapter simply denies the action, the agent loses the opportunity to proceed even after approval.
The project needed a pattern that lets the adapter deny immediately but still allow the action later if approved.

## Decision drivers

- The adapter must respond within the host's hook timeout.
- An approved action that was initially denied must be retryable.
- The approval must be bound to the original action request, not to anything the agent says.

## Considered options

1. Deny with a ticket: deny now, return a ticket, allow retry after approval consumes a single-use grant.
2. Block the hook until approval (waiting past the host timeout).
3. Skip the check entirely for risky actions (fail open).

## Decision outcome

Chosen option: "Deny with a ticket", because it respects the host's timeout while preserving the ability to allow after approval.

- When a host cannot wait, the adapter denies the action and returns an approval ticket.
- The ticket encodes the action request digest.
- If the agent retries the same action after an approver allows it, the retry consumes a single-use grant and is allowed.
- The adapter never trusts the agent's replay of the action; it recomputes the digest and checks for a matching grant.

### Consequences

- Good, because the host's timeout is respected.
- Good, because the action can still proceed after approval.
- Bad, because the agent must retry the exact same action; slight variations produce a new request with no matching grant.

### Confirmation

- The [approval protocol specification](../spec/approval-protocol.md) defines the ticket format and the retry flow.
- Integration pages document each host's timeout and retry behavior.

## Pros and cons of the options

### Deny with a ticket

- Good, because it fits within host timeouts.
- Bad, because it requires the agent to retry.

### Block until approval

- Good, because the agent does not need to retry.
- Bad, because it exceeds host timeouts and may hang the agent.

### Skip the check

- Good, because it never blocks.
- Bad, because it fails open, violating fail-closed ([ADR-0013](0013-fail-closed-failure-semantics.md)).

## More information

The glossary defines [deny-with-ticket](../glossary.md#deny-with-ticket) and [grant](../glossary.md#grant). The approval protocol specification defines the ticket's cryptographic binding.
