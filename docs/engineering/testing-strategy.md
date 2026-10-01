<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Testing strategy

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's testing rules.

## Levels

| Level | What it covers | Where |
|---|---|---|
| Unit | Single functions and types | Each crate's `tests/`. |
| Integration | Crate boundaries and the engine | `crates/*/tests/`. |
| Conformance | The test vectors | `docs/spec/test-vectors/`. |
| Fault injection | Crashes and timeouts in the hook | `fuzz/`. |
| Platform | One supported platform each | CI jobs. |

## Test vectors

Every test vector in `docs/spec/test-vectors/` must pass before a milestone is considered complete.

## Fuzzing

Fuzz targets live in `fuzz/`, a separate nightly workspace.
They run in CI on every pull request from v0.1.

## Miri

Miri runs the engine crates' tests on a nightly toolchain in CI.

## Property tests

The approval state machine is property-tested from v0.3.

## Read next
- [Workspace and crates](workspace-and-crates.md)
- [Continuous integration](ci.md)