<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0028: Deny-with-ticket for hosts that cannot wait

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Approval protocol](../spec/approval-protocol.md), [Deny-with-ticket](../glossary.md#deny-with-ticket)

## Decision outcome

For a host that cannot wait for an approver, the adapter denies the action now and returns an approval ticket.
If the agent retries after an approver approves, the retry consumes a single-use grant.

## Consequences
- Hosts that cannot wait still fail closed.
- The ticket carries the approval request id.
