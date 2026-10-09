<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0037: Unsafe code policy

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Unsafe code policy](../engineering/unsafe-policy.md), [Workspace and crates](../engineering/workspace-and-crates.md), [ADR-0002](0002-rust-for-product-and-tooling.md), [ADR-0038](0038-supply-chain-policy.md)

## Context and problem statement

Shiin parses untrusted input from agents and hosts, including shell commands, file paths, and tool payloads.
Unsafe Rust disables compiler checks that the project relies on to guard these paths.
A memory-safety bug in Shiin could change a decision from `deny` to `allow`, completely defeating the authorization layer.

## Decision drivers

- Unsafe code must be minimized and contained.
- Every unsafe block must be justified and reviewed.
- The workspace lints must not be overridable by individual crates.

## Considered options

1. Forbid `unsafe_code` in all crates except a single `shiin-platform` crate.
- `unsafe_code = "forbid"` in `[workspace.lints.rust]`.
- Only `shiin-platform` is allowed `unsafe`, and only where no safe crate provides the needed facility.
- Every `unsafe` block has a `// SAFETY:` comment, one unsafe operation per block, and tests on every supported platform.
- `shiin-platform` declares its own lint table (copied from the workspace, with `unsafe_code` changed to `deny`).
- Soundness bugs are treated as vulnerabilities through [ADR-0006](0006-vulnerability-reporting.md), not as ordinary bugs.

### Consequences

- Good, because unsafe code is confined to one small, heavily reviewed crate.
- Good, because `forbid` cannot be overridden by `allow` or `expect` inside a crate.
- Bad, because creating `shiin-platform` adds review overhead; it is created only when a concrete need exists.

### Confirmation

- The [unsafe code policy page](../engineering/unsafe-policy.md) and `Cargo.toml` enforce the rule.
- Two-maintainer review is required for any unsafe change.
- Miri runs `shiin-platform` tests in CI.

## Pros and cons of the options

### Forbid unsafe, one exception crate

- Good, because unsafe is minimized and contained.
- Bad, because it requires `shiin-platform` for any OS facility not covered by safe crates.

### Allow unsafe everywhere with review

- Good, because it is flexible.
- Bad, because it distributes unsafe code across many crates and reviewers.

### No unsafe at all

- Good, because it is simplest.
- Bad, because some OS facilities (Windows named pipes, raw syscalls) require it.

## More information

The [unsafe code policy page](../engineering/unsafe-policy.md) is the detailed rulebook. The [supply chain policy ADR](0038-supply-chain-policy.md) covers unsafe code in dependencies.
