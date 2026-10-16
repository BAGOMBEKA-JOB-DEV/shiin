<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-16 -->

# Testing strategy

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's testing rules.

## Levels

| Level | What it covers | Where |
|---|---|---|
| Unit | Single functions and types | In-module `tests` modules. |
| Integration | Crate boundaries and the engine | `crates/*/tests/`. |
| Conformance | The test vectors | `docs/spec/test-vectors/`. |
| Fault injection | Crashes, timeouts, damaged state, and failed audit writes | CLI unit and integration tests. |
| Platform | One supported platform each | CI jobs. |

## Test vectors

Every test vector in `docs/spec/test-vectors/` must pass before a milestone is considered complete.
The CLI evidence tests also sign, encrypt, verify, decrypt, and replay each vector, using fixture keys without a live OS keyring.
Client integration tests connect to the actual Unix daemon service over a local socket to check both sides of the JSON-RPC contract.
`cargo xtask intent-evaluation` reports the task-labelled corpus against a policy-only baseline and fails if a decision or reason disagrees with its label.

## Fuzzing

Fuzz targets are planned in `fuzz/`, a separate nightly workspace.
That workspace and its CI jobs do not exist yet; ordinary fault-injection tests do not establish fuzzing coverage.

## Miri

Miri coverage on a dated nightly is planned and is not currently configured in CI.

## Property tests

The Unix approval state machine has transition-sequence tests for expiry, revocation, binding, and single-use grant consumption.
Cross-platform service tests and the independent security review remain open.

## Read next

- [Workspace and crates](workspace-and-crates.md)
- [Continuous integration](ci.md)
