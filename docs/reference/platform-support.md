<!-- shiin-doc: kind=reference status=draft implementation=n/a milestone=n/a reviewed=2026-10-18 -->

# Platform support

> [!NOTE]
> **Reference: draft.**
> The first alpha provides Linux and macOS CLI and Unix-daemon archives; Windows remains source-only.
> See the limitations below before relying on path matching on Windows.

## Tiers

| Tier | Platforms | Support |
|---|---|---|
| Tier 1 | Linux x86_64, macOS 13+ (x86_64 and arm64) | Tested in CI. |
| Tier 2 | Windows 10+ (x86_64) | Builds; promoted toward Tier 1 in v0.6. |
| Tier 3 | Other architectures | Not guaranteed to build. |

## Before 1.0

The MSRV is pinned stable minus 2 minor versions.
After 1.0, library crates track latest stable minus 6 minor versions.

## What Tier 2 runs today

Windows is Tier 2 because path canonicalization there is not specified yet
([ADR-0019](../adr/0019-path-canonicalization.md) defers the `\\?\` prefix, 8.3 short names,
junctions, and case rules to v0.6).

Continuous integration therefore runs the whole test suite on Windows except the tests whose
expectation depends on a policy glob matching a canonicalized path. Those are marked
`#[cfg(unix)]` in `crates/shiin-cli/tests/hook.rs`. What still runs on Windows is the
fail-closed behaviour that does not involve paths: an unparsable policy, an unknown policy
key, an unwritable audit log, a malformed or empty payload, a tampered audit log, an opaque
command, and the command-line tooling.

This is a real gap, not a formality: until those rules exist, a policy on Windows cannot be
trusted to match the paths an operator wrote. Promotion to Tier 1 means specifying them,
removing the gates, and seeing the full suite pass.

## Read next

- [Toolchain and MSRV](../engineering/toolchain-and-msrv.md)
- [Platform support tiers](../adr/0040-platform-support-tiers.md)
