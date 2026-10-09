<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0039: Release tooling

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Release engineering](../engineering/release-engineering.md), [ADR-0002](0002-rust-for-product-and-tooling.md), [ADR-0017](0017-json-wire-format-and-schemas.md), [ADR-0031](0031-dsse-ed25519-receipts.md)

## Context and problem statement

Shiin ships signed release binaries with a software bill of materials.
The release process must generate checksums, SBOMs, signatures, and changelog entries, all reproducibly.
The project needed a release tooling strategy that works within the Rust ecosystem and the DCO-only contribution model.

## Decision drivers

- Release artifacts must be signed and checksummable.
- The changelog must be generated from commits (Conventional Commits).
- SBOMs must be produced for each binary.

## Considered options

1. A release-tooling crate in `xtask`, plus `cargo-release` or a custom script, using GitHub Actions.
2. Hand-crafted release scripts in shell.
3. A third-party release service (for example, Release Please).

## Decision outcome

Chosen option: "xtask-managed release with cargo-release or equivalent, GitHub Actions for CI", because it keeps the release logic in Rust and the workflow configuration standard.

- Release artifacts are built with `cargo build --release` for all Tier 1 platforms.
- Each binary is signed with Ed25519 and has a checksum file.
- An SBOM is generated in SPDX format.
- The changelog is generated from Conventional Commits via the release tooling.
- GitHub Actions runs the release workflow with `contents: write` permission.
- This is the only place in CI with write permissions.

### Consequences

- Good, because releases are reproducible and the tooling is in Rust.
- Good, because the changelog is derived from commits automatically.
- Bad, because the release workflow has write access and must be carefully guarded.

### Confirmation

- The [release engineering page](../engineering/release-engineering.md) documents the full process.
- Release binaries are signed with the signing key from [ADR-0032](0032-signing-key-storage.md).

## Pros and cons of the options

### xtask-managed release

- Good, because it keeps release logic in Rust and is reproducible.
- Bad, because writing release code in Rust is more involved than a shell script.

### Shell scripts

- Good, because they are quick to write.
- Bad, because they are hard to test and maintain across platforms.

### Third-party service

- Good, because it is hands-off.
- Bad, because it adds a dependency on an external service and may not integrate with DCO.

## More information

Pre-release binaries use the `shiin policy validate` TOML checker (planned: v0.1). Post-1.0 releases include an SBOM in SPDX format.
