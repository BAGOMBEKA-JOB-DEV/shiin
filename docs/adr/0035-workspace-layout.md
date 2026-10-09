<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0035: Workspace layout

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Workspace and crates](../engineering/workspace-and-crates.md), [ADR-0034](0034-edition-msrv-and-toolchain.md), [ADR-0037](0037-unsafe-code-policy.md), `Cargo.toml`, `rust-toolchain.toml`

## Context and problem statement

As the project grows from one crate to many, the repository layout must keep the dependency graph acyclic, the crate boundaries clear, and the build efficient.
A virtual workspace root with product crates in subdirectories is the standard Rust layout, but conventions vary.

## Decision drivers

- No crate at the root should be privileged.
- Product crates should be grouped and clearly named.
- Repository tooling should be separate and never published.
- Fuzzing should not pollute the main workspace.

## Considered options

1. Virtual workspace root, `crates/<name>/` for products, `xtask/` for tooling, `fuzz/` for fuzzing.
2. Root package with all crates as sub-crates.
3. A monorepo with flat crate directories at the root.

## Decision outcome

Chosen option: "Virtual workspace root, crates/ for products, xtask/ for tooling, fuzz/ for fuzzing", because it avoids privileging a root crate and groups concerns clearly.

- The root `Cargo.toml` is a virtual manifest with no `[package]` section.
- Product crates live in `crates/<name>/`, where the directory name is the crate name.
- `xtask/` provides repository automation (`cargo xtask docs-check`) and sets `publish = false`.
- `fuzz/` is a separate workspace with its own nightly toolchain for fuzz targets.
- Every member inherits `edition`, `rust-version`, `license`, `repository`, and `authors` from `[workspace.package]`.
- Every member sets `[lints] workspace = true`, except `shiin-platform`.

### Consequences

- Good, because no crate is privileged by living at the root.
- Good, because crates are grouped logically.
- Good, because `xtask` and `fuzz` are isolated from the published crates.
- Bad, because `crates/<name>/` adds a directory level; contributors must learn the layout.

### Confirmation

- `Cargo.toml` and `.cargo/config.toml` implement this layout.
- The [workspace and crates page](../engineering/workspace-and-crates.md) documents the full crate list.

## Pros and cons of the options

### Virtual workspace root

- Good, because no crate is privileged.
- Good, because the layout is the Rust community standard.
- Bad, because it adds a directory level for product crates.

### Root package

- Good, because it is simple for a single-crate project.
- Bad, because it privileges one crate and does not scale to many crates.

### Flat directories

- Good, because it avoids nested directories.
- Bad, because it mixes product and tooling crates without clear boundaries.

## More information

The [workspace and crates page](../engineering/workspace-and-crates.md) is the single source of truth for the crate list and their responsibilities.
