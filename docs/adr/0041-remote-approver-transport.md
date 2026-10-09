<!-- shiin-doc: kind=adr status=deferred implementation=n/a milestone=v0.6 reviewed=2026-09-14 -->

# ADR-0041: Remote approver transport

> [!NOTE]
> **Architecture decision: deferred.**
> Deferred to v0.6. Do not implement until confirmed by an RFC.

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Approval protocol](../spec/approval-protocol.md), [ADR-0027](0027-approval-resolutions-and-grants.md), [Roadmap](../project/roadmap.md#v06-reach)

## Context and problem statement

The v0.3 approval protocol supports local approvers (command-line, desktop, editor).
For teams or headless environments, approvals may need to reach a remote approver through a webhook or API.
The transport mechanism for remote approvals must be secure, authenticated, and replay-resistant.

## Decision drivers

- Remote approval must not expose sensitive action data to an untrusted channel.
- Approvals must be non-repudiable.
- The transport must be opt-in, consistent with the no-telemetry principle.

## Considered options

1. Signed webhooks with Ed25519, delivered over HTTPS with mutual TLS.
2. A centralized approval server with its own identity model.
3. Polling-based approval via a REST API.

## Decision outcome

Chosen option: "Deferred pending RFC", because the threat model for remote approvals and the exact transport are not yet settled.

- This ADR is deferred to v0.6.
- The [approval protocol](../spec/approval-protocol.md) is designed to be transport-independent so a future RFC can add remote transport without changing the approval message format.
- The local-first principle ([ADR-0007](0007-no-telemetry.md)) and local-first deployment ([ADR-0022](0022-local-first-daemonless-deployment.md)) apply: remote approval is opt-in, never default.

### Consequences

- Good, because deferring avoids committing to a transport that may be superseded.
- Bad, because teams needing remote approvals today have no path.

### Confirmation

- The RFC process will produce a proposal for remote approval transport.
- The approval protocol specification must remain transport-agnostic.

## Pros and cons of the options

### Signed webhooks over HTTPS

- Good, because it is standard and replay-resistant with timestamps.
- Bad, because it requires HTTPS infrastructure and mutual TLS management.

### Centralized server

- Good, because it is a simple model.
- Bad, because it creates a central point of failure and a data-collection risk.

### REST polling

- Good, because it works behind firewalls.
- Bad, because it is slower and harder to authenticate.

## More information

The [roadmap v0.6 section](../project/roadmap.md#v06-reach) lists remote approvals. This ADR must be `accepted` by an RFC before implementation begins.
