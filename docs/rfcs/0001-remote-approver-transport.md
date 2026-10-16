<!-- shiin-doc: kind=rfc status=draft implementation=n/a milestone=n/a reviewed=2026-10-16 -->

# RFC-0001: Remote approver transport

> [!NOTE]
> **RFC: draft.**
> This proposal is not accepted. Remote-approval implementation remains blocked on maintainer review and an accepted decision record.

- **Date:** 2026-10-16
- **Status:** draft
- **Supersedes:** none
- **Superseded by:** none

## Summary

This RFC proposes an opt-in outbound HTTPS polling transport for remote approvals.
It replaces the v0.6 roadmap's unspecified "signed webhooks" wording with a transport that does not require an inbound listener on the developer's machine.
Every response is signed by a configured remote approval key and bound to one pending ticket, request binding, nonce, and expiry.

## Motivation

The local daemon currently requires an operator terminal to resolve a pending action.
Headless and team environments need a path for a human to review that action remotely.
The existing [ADR-0041](../adr/0041-remote-approver-transport.md) defers implementation until the transport and its threat model are settled.

## Proposal

- Remote approval is disabled unless the operator configures an HTTPS endpoint and a trusted Ed25519 response key.
- The local daemon sends only pending approval requests to that endpoint and polls for a response.
  It does not open a listener or accept an unsolicited remote connection.
- The outbound request includes the existing ticket and normalized action summary, a digest of the bound request, a fresh random nonce, an audience identifier, and a short expiry.
  It excludes operator credentials, signing keys, replay keys, and unrelated audit history.
- The remote response is a signed, versioned envelope that binds the endpoint audience, ticket, request digest, nonce, `approver_id`, resolution, optional grant scope, lifetime, issue time, and expiry.
- The daemon verifies the configured key, signature, audience, ticket, request digest, nonce, resolution, and time bounds before passing the response to the existing approval state machine.
  The state machine still re-evaluates current policy and task state, and a grant never overrides `deny`.
- HTTPS certificate validation is mandatory.
  Redirects are rejected, request and response sizes are bounded, and endpoint configuration is operator-owned rather than workspace-owned.
- Polling failures, invalid signatures, stale responses, missing nonces, unknown fields, and expired requests leave the action `pending` or deny it according to the existing attended/unattended behavior.
  None may produce `allow`.
- The transport remains opt-in and performs no background network activity when no endpoint is configured.

## Considered alternatives

- **Inbound signed webhooks:** they can deliver decisions promptly but require a remotely reachable callback and enlarge the daemon's network attack surface.
- **Unsigned REST polling:** it avoids inbound connections but does not provide a durable, independently verifiable approver statement.
- **A centralized Shiin approval service:** it moves identity, availability, and data custody into a hosted service, which conflicts with the current local-first deployment model.

## Consequences

- A signed response can prove that a configured key approved a specific pending request, but it cannot prove the remote service authenticated a human correctly.
- Operators must protect the remote key and review which action fields leave the machine.
- Polling adds latency and depends on both the remote endpoint and the local network.
- The approval protocol remains transport-independent; this proposal adds a transport profile without changing its existing resolution values.
- Threat-model review must cover endpoint trust, sensitive action disclosure, key rotation, DNS rebinding, response replay, and compromised remote services before implementation.

## Confirmation

Before implementation, maintainers must accept or revise this RFC and replace deferred ADR-0041 with an accepted ADR.
Conformance tests must prove that wrong keys, altered payloads, stale tickets, mismatched bindings, reused nonces, redirects, timeouts, and malformed responses never produce `allow`.
The transport must also pass an independent threat-model review.
