<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-15 -->

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

## Fuzzing

Fuzz targets are planned in `fuzz/`, a separate nightly workspace.
That workspace and its CI jobs do not exist yet; ordinary fault-injection tests do not establish fuzzing coverage.

## Miri

Miri coverage on a dated nightly is planned and is not currently configured in CI.

## Property tests

The planned approval state machine will have property tests in v0.3.

## Read next

- [Workspace and crates](workspace-and-crates.md)
- [Continuous integration](ci.md)
