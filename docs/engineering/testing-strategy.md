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
`cargo xtask conformance` packages the core vectors, schemas, and a revision-bound result report; the documentation workflow publishes the bundle as a downloadable artifact.
The CLI evidence tests also sign, encrypt, verify, decrypt, and replay each vector, using fixture keys without a live OS keyring.
Client integration tests connect to the actual Unix daemon service over a local socket to check both sides of the JSON-RPC contract.
`cargo xtask intent-evaluation` reports the task-labelled corpus against a policy-only baseline and fails if a decision or reason disagrees with its label.
`cargo xtask dependency-check` rejects known network stacks and async runtimes in the normal transitive dependency graph of the four engine crates.
CI separately checks the isolated engine for WebAssembly and tests the schema crate without OS clock/randomness functions.
The hosted release measurements also exercise 100/500-rule policies and hook calls against a thousand-event audit log, checking every observed latency against the existing budgets.

## Fuzzing

`fuzz/` is a separate nightly workspace with wire-request, classification, and authorization targets.
The wire target parses and validates action requests and checks that validated records survive a serialization round trip.
The classification target exercises shell tokenization and verifies that unsupported SQL syntax stays opaque.
The authorization target exercises deterministic decisions for validated arbitrary requests, unattended non-pending behavior, and refusal when capabilities are absent or explicitly denied.
It uses a fixed compiled policy and re-evaluates the gate with stricter capability inputs; the seed requests come from the full golden corpus.
`cargo xtask fuzz-smoke` seeds the targets from published action vectors and representative command/SQL inputs, then runs each for twenty seconds with a 64-KiB input bound and a 1-GiB resident-memory limit.
The hosted robustness workflow runs this command on `nightly-2026-10-15`; no fuzz build is required on a developer's laptop.
These bounded smoke runs do not establish exhaustive parser or engine security coverage.

## Miri

The hosted robustness workflow interprets the schema tests without runtime clocks or randomness on `nightly-2026-10-15`.
This covers pure wire validation and timestamp formatting, not native Cedar, TLS, OS key stores, or IPC.

## Native key stores

`cargo xtask native-keyring-check` runs an explicitly opted-in lifecycle test on ephemeral CI runners.
It exercises the actual platform backend through the same private store implementation as production, using a random `shiin-EXAMPLE-native-test-` service namespace.
The production service remains fixed at `shiin`; no environment setting or public API changes its namespace.
The test checks persistence, signing and verification, rotation, retained-key replay, wrong-key rejection, deleted keys, and corrupt selected entries without implicit replacement.
Cleanup deletes only entries created in the isolated namespace.
Ordinary workspace tests skip this test, so they do not access an operator's key store.

The Linux job starts a temporary D-Bus session and unlocks an isolated runner's Secret Service with an explicit `EXAMPLE` fixture password through stdin.
The daemon's foreground and unlock behavior follows its [manual](https://manpages.debian.org/unstable/gnome-keyring/gnome-keyring-daemon.1.en.html) (verified 2026-10-16).
The Rust task owns and stops the test daemon.
macOS and Windows jobs use their runner's native backend.
These tests do not establish protection against arbitrary unmediated code running as the same operating-system user.

## Property tests

The Unix approval state machine has transition-sequence tests for expiry, revocation, binding, and single-use grant consumption.
Cross-platform service tests and the independent security review remain open.

## Read next

- [Workspace and crates](workspace-and-crates.md)
- [Continuous integration](ci.md)
