<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0038: Supply chain policy

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Workspace and crates](../engineering/workspace-and-crates.md), [ADR-0037](0037-unsafe-code-policy.md), [ADR-0034](0034-edition-msrv-and-toolchain.md), [ADR-0039](0039-release-tooling.md)

## Context and problem statement

Shiin depends on third-party crates, some of which contain `unsafe` code.
A vulnerable or compromised dependency could let an attacker change a decision or extract data.
The project needed a supply-chain policy that admits dependencies only after review.

## Decision drivers

- Dependencies must be declared once at the workspace level with default features off.
- Unsafe usage in dependencies must be weighed against safe alternatives.
- Security advisories in the RustSec database must be monitored.

## Considered options

1. Declare all dependencies in `[workspace.dependencies]` with `default-features = false`, and review each for safety and audit history.
2. Allow per-crate dependency declarations without central review.
3. Vet every dependency through cargo-vet before use.

## Decision outcome

Chosen option: "Workspace-level declaration with default features off and review-based admission", because it gives visibility and control without requiring a vet database from day one.

- Every third-party dependency is declared in `[workspace.dependencies]` with a version requirement and `default-features = false`.
- Members reference it with `workspace = true` and enable only the features they use.
- Admission criteria include: amount of unsafe code, documentation of unsafe blocks, RustSec advisory history, and whether a safe alternative exists.
- `cargo-deny` checks the dependency tree in CI (planned: v0.4).

### Consequences

- Good, because all dependencies are visible in one place.
- Good, because default features are off, reducing the attack surface.
- Bad, because the review process adds latency to adopting new dependencies.

### Confirmation

- `Cargo.toml` declares dependencies in `[workspace.dependencies]`.
- CI runs `cargo-deny` (planned: v0.4).

## Pros and cons of the options

### Workspace-level declaration with review

- Good, because it provides visibility and control.
- Bad, because it adds review overhead.

### Per-crate declarations

- Good, because it is simpler.
- Bad, because dependencies are scattered and hard to audit.

### cargo-vet

- Good, because it is thorough.
- Bad, because it requires maintaining a vet database, which is heavy for a pre-alpha project.

## More information

The [workspace and crates page](../engineering/workspace-and-crates.md) documents the dependency rules. The unsafe code policy ADR covers unsafe code in dependencies.
