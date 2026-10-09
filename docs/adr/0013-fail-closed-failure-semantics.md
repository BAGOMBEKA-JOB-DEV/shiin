<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0013: Fail-closed failure semantics

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Security model](../security/security-model.md), [Error handling](../engineering/error-handling.md), [ADR-0009](0009-deterministic-enforcement.md), [ADR-0011](0011-decision-values-and-combining.md), [Glossary](../glossary.md#fail-closed)

## Context and problem statement

When the engine, adapter, or policy itself has a problem — an error, a timeout, an unreadable policy file — there is no decision to return.
The alternative is either to let the action through (fail open) or to refuse it (fail closed).
An authorization layer that fails open defeats its own purpose.

## Decision drivers

- A crash or misconfiguration must never let an unauthorized action proceed.
- Operators must notice failures, not silently accept them.
- The decision must be fast: the host's hook timeout is measured in seconds.

## Considered options

1. Fail closed: any failure produces a `deny` decision.
2. Fail open: any failure lets the action proceed.
3. Fail to `pending`: any failure waits for human approval.

## Decision outcome

Chosen option: "Fail closed", because an authorization layer that lets actions through on its own failure is worse than useless.

- Every operational error in the hook binary, the daemon, or the engine is routed to a single conversion function that produces a `deny` decision.
- Errors and timeouts produce `deny`, as stated in [How Shiin works](../concepts/how-shiin-works.md).
- A panic in the hook binary installs a panic hook that writes the host's deny response before unwinding.
- A failed audit write for a non-read action also produces `deny`, as specified in [ADR-0030](0030-audit-durability.md).

### Consequences

- Good, because a failure can never authorize an action.
- Good, because failures are visible and diagnosable.
- Bad, because a misconfigured policy can stop all work; the operator must be able to read the logs to diagnose it.

### Confirmation

- Every hook, daemon, and MCP proxy entry point routes `Err` to the single conversion function.
- A fault-injection test makes the hook panic and time out and asserts the result is `deny` (exit criterion: v0.1).

## Pros and cons of the options

### Fail closed

- Good, because it preserves the security guarantee under failure.
- Bad, because it can block legitimate work when misconfigured.

### Fail open

- Good, because it keeps work flowing.
- Bad, because it defeats the purpose of the authorization layer.

### Fail to pending

- Good, because a human can override.
- Bad, because in CI or unattended mode there is no human, and `pending` becomes a hang or a silent denial.

## More information

The [security model](../security/security-model.md) lists the failure modes and their resulting decisions. The error-handling page describes the code-level implementation.
