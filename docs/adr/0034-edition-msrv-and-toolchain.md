<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0034: Edition, MSRV, and toolchain

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Toolchain and MSRV](../engineering/toolchain-and-msrv.md), [Workspace and crates](../engineering/workspace-and-crates.md), [ADR-0002](0002-rust-for-product-and-tooling.md), `rust-toolchain.toml`, `Cargo.toml`

## Context and problem statement

Rust evolves quickly. A pinned toolchain ensures reproducible lint and formatting results, but contributors must be able to build with a slightly older compiler.
The project needed to pick an edition, a pinned toolchain, and a minimum supported Rust version.

## Decision drivers

- CI and contributor builds must be reproducible.
- The MSRV must not be too aggressive, so dependency graphs can support it.
- The edition must be current enough for the features the project needs.

## Considered options

1. Edition 2024, pinned toolchain 1.98.1, MSRV 1.96.
2. Edition 2021, pinned toolchain 1.85, MSRV 1.83.
3. Track latest nightly for cutting-edge features.

## Decision outcome

Chosen option: "Edition 2024, pinned toolchain 1.98.1, MSRV 1.96", because Edition 2024 is the current edition and the pinned/nearby versions are stable and widely supported.

- Edition 2024, set in `[workspace.package].edition` in `Cargo.toml`.
- Pinned toolchain 1.98.1, set in `rust-toolchain.toml`.
- MSRV 1.96, set in `[workspace.package].rust-version` in `Cargo.toml` (pinned stable minus 2 minor versions).
- `rustfmt` style edition is 2024, set in `rustfmt.toml`.
- Dependency resolver 3, which is MSRV-aware.

### Consequences

- Good, because the toolchain is reproducible across contributors and CI.
- Good, because the MSRV is achievable by most current Rust installations.
- Bad, because newer compiler features are unavailable until the MSRV is raised.

### Confirmation

- `rust-toolchain.toml` and `Cargo.toml` declare the versions.
- The [toolchain and MSRV page](../engineering/toolchain-and-msrv.md) documents the policy for bumping them.

## Pros and cons of the options

### Edition 2024, toolchain 1.98.1, MSRV 1.96

- Good, because it uses the current edition and recent stable.
- Bad, because contributors on older Rust must upgrade.

### Edition 2021, older toolchain

- Good, because it supports more legacy environments.
- Bad, because it forgoes Edition 2024 features and is less future-proof.

### Track nightly

- Good, because it enables cutting-edge features.
- Bad, because nightly is unstable and breaks reproducibility.

## More information

Changing the edition, MSRV, or toolchain rules requires a superseding ADR. The [unsafe code policy ADR](0037-unsafe-code-policy.md) sets the baseline on which these rules build.
