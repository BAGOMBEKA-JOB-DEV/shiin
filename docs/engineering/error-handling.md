<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# Error handling

> [!NOTE]
> **Project policy: draft.**
> How Shiin's crates represent, report, and recover from failures.
> No product crate exists yet; these rules apply from the first one.

Shiin guards actions for an untrusted [agent](../glossary.md#agent).
Its error handling therefore has one overriding goal: a failure must never let an action through.
This page follows [ADR-0036](../adr/0036-error-handling.md),
and the resulting failure semantics are specified in the [security model](../security/security-model.md) and [ADR-0013](../adr/0013-fail-closed-failure-semantics.md).

## Taxonomy

Every unhappy outcome belongs to exactly one of four categories.

| Category | Examples | Representation | What happens |
|---|---|---|---|
| Decisions | A rule denies a write; an action needs approval | A `Decision` value, returned normally | The adapter enforces it. It is never an `Err`. |
| Operational errors | An unreadable policy file, a failed audit write, a malformed host payload, an IPC timeout | A per-crate `thiserror` enum marked `#[non_exhaustive]` | Mapped to a fail-closed `deny` in one place, logged with its error code |
| User diagnostics | A syntax error in `policy.toml`, a shadowed rule | Diagnostic data with source spans, rendered by the CLI with `miette` | Shown to the operator; at run time, an invalid policy still fails closed |
| Bugs | A violated internal invariant | `debug_assert!` in development builds; no panics on the engine hot path | Fixed in code. If a hook panics anyway, its panic hook answers `deny`. |

### Decisions are not errors

A [`deny`](../glossary.md#deny) is a successful evaluation that reached a negative answer.
It is returned as `Ok`, or as a plain `Decision`, never as an `Err`.
This keeps `?` from accidentally turning an authorization result into an error path, and it keeps error handling free to treat every `Err` as a failure.
A `pending` decision is also a decision, not an error.

### Operational errors

- Each library crate defines its own error enum with [`thiserror`](https://github.com/dtolnay/thiserror).
  A large crate may define one enum per major module.
- Error enums are `#[non_exhaustive]`, so adding a variant is not a breaking change.
- Variants carry structured context, such as the path or key involved, not preformatted strings.
- An error that wraps another marks it with `#[source]` or `#[from]` and does not repeat the inner message in its own,
  so reporting the error chain does not print the same text twice.
- Variants never carry secret values such as file contents, tokens, or key material.
  Paths, keys, identifiers, and digests are allowed.
- Libraries do not use `anyhow`, `eyre`, or `Box<dyn Error>` in their APIs.
  Callers must be able to match on errors, and every error must keep its stable code.

### User diagnostics

- Errors in operator-written input, above all policy files, are reported with the exact location and a suggested fix.
- `shiin-policy` exposes diagnostics as plain data: an error code, a message, source spans, and optional help text.
- `shiin-cli` renders that data with [`miette`](https://github.com/zkat/miette), showing the offending lines of the policy file.
  Keeping `miette` in the binary keeps rendering concerns out of the library.
- Commands such as `shiin policy validate` report every diagnostic they find, not only the first.
- Diagnostics are for the [operator](../glossary.md#operator).
  They never reach the agent.

### Bugs

- The *engine hot path* is `shiin-engine`'s evaluation function and everything it calls.
  It contains no `unwrap`, `expect`, `panic!`, unchecked indexing, or unchecked arithmetic, enforced by the lints in [Coding standards](coding-standards.md).
- `debug_assert!` is fine for documenting internal invariants.
  It is compiled out of release builds and turns invariant violations into test and fuzzing failures.
- A condition that untrusted input can trigger is never only asserted.
  It is handled as an operational error.
- Outside the hot path, `expect` is allowed where an invariant makes failure impossible, with a lint suppression naming the invariant.

## Errors become decisions in exactly one place

An operational error that occurs while handling an action must end in `deny`.
That conversion is implemented once:
a single function in `shiin-engine` (shown below as `deny_on_failure`, a placeholder name) is the only code that constructs a `deny` decision for a failure.

- The hook binary, the daemon, and the MCP proxy route every `Err` to that function at their outermost request handler.
  None of them builds its own failure decision.
- The engine uses the same function for failures inside evaluation, so evaluation itself never returns an `Err`.
- The resulting decision carries the failure reason code defined in [Decision](../spec/decision.md).
  The operational error's code is recorded in the audit event and the log, not in the agent-facing message.
- Failures that happen after a decision but before the adapter answers, such as a failed audit write for a non-read action, also go through this function,
  so the answer becomes `deny` ([ADR-0030](../adr/0030-audit-durability.md)).

A single conversion point means there is one place to review, test, and fuzz,
and no code path can invent a different, more permissive answer to a failure.

## Error codes

- Every operational error variant and every diagnostic has a stable code, for example `SHIIN-POL-0003`.
- Codes are listed, with their meaning and remedy, in [Error codes](../reference/error-codes.md), which is the only home of the list.
- A code is never renumbered or reused, even after the error it named is removed.
- Codes appear in CLI output, logs, and audit events, so a report can be matched to documentation without the full message.
- A planned test checks that every code used in the source appears in the reference and that every code in the reference is used or marked retired.

## Message style

- Messages start with a lowercase letter and have no trailing period: `failed to read policy file "/home/dev/project/.shiin/policy.toml"`.
- Proper nouns and code identifiers keep their case.
- A message says what failed, not how to fix it.
  Fixes go in diagnostic help text or in the error-code reference.
- Messages are concise and avoid words such as "error" or "fatal", which the reporting context already conveys.

## What the agent sees

Internal errors never reach the untrusted agent verbatim.

| Audience | What it receives |
|---|---|
| Agent, through the host | The `deny` decision and the fixed agent-facing message for failures defined in [Decision](../spec/decision.md). No error text, path, error code, or stack trace. |
| Operator | CLI diagnostics, the error code, and the full error chain in logs |
| Audit log | An event recording the failure reason code and the error code, without secret values |

Error messages can reveal file layout, policy structure, or which check failed,
which an agent could use to probe for a bypass.
The [threat model](../security/threat-model.md) treats deny-reason oracles as an in-scope threat.

## The hook binary's panic hook

Hosts treat a crashed hook as permission to proceed, so a panic in the hook binary must still produce `deny`.

- The first statement of the hook binary's `main` installs a panic hook with `std::panic::set_hook`, before any input is read.
- The panic hook writes the host's deny response and exits with the host's deny exit status.
  The response encoding for each host is defined on that host's [integration page](../guides/integrations/claude-code.md) and in the [adapter contract](../spec/adapter-contract.md).
- The response is a fixed, pre-encoded byte string prepared at start-up.
  The panic hook does not allocate new buffers, take locks that the panicking code might hold, or call code that can panic again.
- The panic hook runs before unwinding or aborting, so it works with either panic strategy.
- A separate deadline watchdog answers `deny` before the host's hook timeout, as the adapter contract requires.

The panic hook does not cover every crash.
A process killed by a signal, the operating system's out-of-memory handler, or a stack overflow exits without running it,
and some hosts treat that exit as permission to proceed.
The hook path therefore avoids unbounded recursion and unbounded allocation driven by untrusted input,
and the residual risk is recorded in the [security model](../security/security-model.md).
A fault-injection test that makes the hook panic and time out is an exit criterion for v0.1; see [Testing strategy](testing-strategy.md).

## Illustrative example

The following Rust is illustrative only.
The crates do not exist yet, and every name and error code in it is a placeholder.

```rust
// Illustrative only: not real Shiin code. Names and codes are placeholders.

use std::path::PathBuf;

/// Errors from loading and compiling a policy.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PolicyError {
    /// The policy file could not be read.
    #[error("failed to read policy file {path:?}")]
    Read {
        /// The file that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The policy uses a key that the policy syntax does not define.
    #[error("unknown key `{key}` in policy")]
    UnknownKey {
        /// The unrecognized key.
        key: String,
    },
}

impl PolicyError {
    /// Returns the stable code listed in the error-code reference.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::Read { .. } => ErrorCode::new("SHIIN-POL-0001"),
            Self::UnknownKey { .. } => ErrorCode::new("SHIIN-POL-0003"),
        }
    }
}

/// The hook binary's outermost request handler.
///
/// Every failure is routed to the single conversion function; nothing else
/// constructs a failure decision.
fn handle(input: &[u8], state: &HookState) -> Decision {
    match evaluate_hook_input(input, state) {
        Ok(decision) => decision,
        Err(error) => {
            tracing::error!(code = %error.code(), error = %error, "evaluation failed, denying");
            shiin_engine::deny_on_failure(error.code())
        }
    }
}
```
