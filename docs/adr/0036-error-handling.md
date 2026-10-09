<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0036: Error handling

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Error handling](../engineering/error-handling.md), [ADR-0013](0013-fail-closed-failure-semantics.md), [ADR-0034](0034-edition-msrv-and-toolchain.md), [ADR-0002](0002-rust-for-product-and-tooling.md)

## Context and problem statement

Shiin's code handles untrusted input from agents and hosts, and its error handling must never let a failure let an action through.
At the same time, operators need clear diagnostics to fix configuration errors, and internal bugs must be caught during testing.
The project needed a taxonomy of failure that separates authorization decisions from operational errors and diagnostics.

## Decision drivers

- Decisions (`allow`, `deny`, `pending`) are not errors and never become `Err`.
- Operational errors must produce fail-closed `deny`.
- Diagnostics must be actionable for operators.
- Bugs must be caught in tests, not silently handled.

## Considered options

1. A four-category taxonomy: decisions, operational errors, user diagnostics, and bugs.
2. A single `Result<Decision, Error>` with errors propagated everywhere.
3. Panic on any unexpected condition and let the host handle it.

## Decision outcome

Chosen option: "Four-category taxonomy", because it keeps concerns separated and ensures failures always produce `deny`.

- **Decisions** (`Ok(Decision)`): `allow`, `deny`, `pending`. Never `Err`.
- **Operational errors** (per-crate `thiserror` enums, `#[non_exhaustive]`): unreadable files, IPC timeouts, malformed payloads. Mapped to fail-closed `deny` by one conversion function.
- **User diagnostics** (with source spans, rendered by `miette`): syntax errors in policy files. Shown to the operator, never to the agent.
- **Bugs** (`debug_assert!`, invariant violations): fail during testing; production failures produce `deny` via the panic hook.

### Consequences

- Good, because the single conversion function means there is one place to review and test.
- Good, because diagnostics are separated from operational errors.
- Bad, because the taxonomy adds upfront design cost.

### Confirmation

- Every operational error routes through the single `deny_on_failure` function.
- A fault-injection test makes the hook panic and asserts the result is `deny`.

## Pros and cons of the options

### Four-category taxonomy

- Good, because it separates concerns and ensures fail-closed.
- Bad, because it is more structured than a single error type.

### Single Result with errors everywhere

- Good, because it is simple.
- Bad, because it conflates decisions with failures and risks letting `?` turn a `deny` into an error path.

### Panic on unexpected conditions

- Good, because it crashes fast.
- Bad, because hosts may treat a crash as permission to proceed.

## More information

The [error handling engineering page](../engineering/error-handling.md) describes the implementation. Each error variant has a stable code listed in [error codes](../reference/error-codes.md).
