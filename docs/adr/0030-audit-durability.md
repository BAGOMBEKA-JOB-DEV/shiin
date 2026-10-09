<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0030: Audit durability

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Error handling](../engineering/error-handling.md), [Audit events](../spec/audit-events.md), [ADR-0013](0013-fail-closed-failure-semantics.md), [ADR-0029](0029-hash-chained-jsonl-audit-log.md)

## Context and problem statement

When an action is allowed, the audit event must be written before the action proceeds.
If the write fails or the process crashes before flushing, the audit log has a gap.
The project needed a durability model that handles write failures as a security event, not a silent skip.

## Decision drivers

- Audit gaps must not be silent.
- The host must not proceed if the audit write fails for a non-read action.
- Performance matters: fsync on every event is expensive.

## Considered options

1. Write the audit event and fsync before the adapter returns `allow`; a write failure produces `deny`.
2. Buffer audit events in memory and flush periodically.
3. Best-effort writes with no failure handling.

## Decision outcome

Chosen option: "Write and fsync before returning allow; failure produces deny", because an audit gap on an allowed action is a security-relevant failure.

- For every non-read action that the engine decides to `allow`, the adapter writes the audit event to the log and flushes it before returning `allow` to the host.
- If the audit write fails, the failure is routed through the same fail-closed conversion function ([ADR-0013](0013-fail-closed-failure-semantics.md)) and the decision becomes `deny`.
- Read actions that are allowed do not require an audit write before returning, but the decision is still recorded.
- The audit log uses buffered writes with periodic flushes for performance, but an explicit flush follows every event that gates a non-read action.

### Consequences

- Good, because audit gaps on non-read actions are impossible without a `deny`.
- Good, because the failure mode is visible and logged.
- Bad, because fsync on every non-read action adds latency to the hook path.

### Confirmation

- The [error handling page](../engineering/error-handling.md) routes audit-write failures through the single conversion function.
- The [adapter contract](../spec/adapter-contract.md) specifies the pre-allowment audit ordering.

## Pros and cons of the options

### Fsync before allow

- Good, because audit events are durable before actions proceed.
- Bad, because fsync adds latency.

### Periodic flush

- Good, because it is fast.
- Bad, because a crash can lose recent events.

### Best-effort only

- Good, because it never blocks.
- Bad, because audit gaps are silent and undetectable.

## More information

Audit events are hash-chained per [ADR-0029](0029-hash-chained-jsonl-audit-log.md). Signed checkpoints and receipts are in [ADR-0031](0031-dsse-ed25519-receipts.md).
