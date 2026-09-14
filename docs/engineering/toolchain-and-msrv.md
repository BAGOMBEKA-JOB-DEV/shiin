<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# Toolchain and MSRV

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's Rust toolchain version and minimum supported Rust version (MSRV).
> The pinned toolchain, edition, and MSRV below are already set in the workspace files.

Shiin pins the exact compiler every contributor and every CI job uses,
and separately promises the oldest compiler its crates build with.
The decision is recorded in [ADR-0034](../adr/0034-edition-msrv-and-toolchain.md).

## Current baseline

| Setting | Value | Where it is set |
|---|---|---|
| Edition | 2024 | `edition` in `[workspace.package]` of `Cargo.toml` |
| Dependency resolver | 3 | `resolver` in `[workspace]` of `Cargo.toml` |
| Pinned toolchain | 1.98.1 | `channel` in `rust-toolchain.toml` |
| MSRV | 1.96 | `rust-version` in `[workspace.package]` of `Cargo.toml` |
| Formatter style edition | 2024 | `style_edition` in `rustfmt.toml` |

## The pinned toolchain

`rust-toolchain.toml` is:

```toml
[toolchain]
channel = "1.98.1"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

- rustup reads this file, so every `cargo` command run inside the repository uses Rust 1.98.1 with `rustfmt` and Clippy.
- CI installs the same toolchain from the same file.
  The documentation workflow in `.github/workflows/docs.yml` already does this.
- Pinning makes lint and formatting results reproducible.
  A new stable release can add Clippy lints, and with warnings denied in CI an unpinned toolchain would break unrelated pull requests on release day.

### Bumping the toolchain

- The toolchain is bumped only by a pull request that changes `rust-toolchain.toml`.
- The same pull request fixes any new warnings, so `main` stays green.
- Bumps normally follow a new stable Rust release within one six-week release cycle, not on its first day,
  so that early point releases are picked up.
- A toolchain bump that also raises the MSRV follows the MSRV rules below.

## Minimum supported Rust version

The MSRV is the oldest Rust release that the workspace's crates are guaranteed to build with.

| Period | Crates | MSRV rule |
|---|---|---|
| Before 1.0 | All crates | Pinned stable minus 2 minor versions |
| After 1.0 | Library crates | Latest stable minus 6 minor versions, about eight months of releases |
| After 1.0 | Binary crates (`shiin`, `shiind`, and other executables) | May track latest stable minus 2 |

Today the pinned stable is 1.98, so the MSRV is 1.96.
The longer window after 1.0 matters most for `shiin-schema` and `shiin-client`,
which other projects embed with their own compiler constraints (see [Workspace and crates](workspace-and-crates.md)).
Most users install binaries prebuilt, so binaries can adopt new compiler features sooner.

### Declaring the MSRV

- The MSRV is declared once, as `rust-version` in `[workspace.package]`, and inherited by every member.
- After 1.0, a binary crate that tracks a newer MSRV overrides `rust-version` in its own manifest.
- Resolver 3 is MSRV-aware.
  When Cargo resolves dependency versions, it prefers versions whose declared `rust-version` is compatible with the workspace's,
  so updating the lockfile does not silently require a newer compiler.
  The resolver can only use what dependencies declare, so the MSRV check below is still needed.

### Raising the MSRV

- An MSRV bump is allowed only in a minor release, never in a patch release.
  Before 1.0, a minor release is a change of the second number, for example 0.3.x to 0.4.0.
- Every MSRV bump is listed as its own item in the changelog of that minor release.
- The pull request explains which language or library feature, or which dependency, needs the newer compiler.
- The MSRV is never raised beyond what the rules in the table allow.

### Checking the MSRV

The MSRV is checked with [cargo-hack](https://github.com/taiki-e/cargo-hack), which runs the check with each package's declared `rust-version`:

```console
$ cargo hack check --rust-version --workspace
```

This check is planned to run in CI on every pull request; see [Continuous integration](ci.md).
It builds only; tests run on the pinned toolchain.

## Nightly Rust

Nightly Rust is used for exactly two things:

- **Fuzzing**, because [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) needs nightly compiler features.
- **[Miri](https://github.com/rust-lang/miri)**, the interpreter that detects undefined behavior in `unsafe` code.

Nightly is never required of contributors.
Building, testing, linting, and checking the documentation all work on the pinned stable toolchain.
CI jobs that need nightly use a dated nightly named in the workflow, updated by pull request, so a nightly regression cannot break them unannounced.
See [Testing strategy](testing-strategy.md) and [Unsafe code policy](unsafe-policy.md).

## Setting up

1. Install rustup by following the instructions at [rustup.rs](https://rustup.rs/).
2. From the repository root, install the pinned toolchain named in `rust-toolchain.toml`, then run a check to confirm the setup:

   ```console
   $ rustup toolchain install
   $ cargo xtask docs-check
   ```

3. Optionally, for fuzzing or Miri, install a nightly toolchain and cargo-fuzz:

   ```console
   $ rustup toolchain install nightly --component miri
   $ cargo install --locked cargo-fuzz
   ```

The documentation tools are installed as described in [Documentation tooling and CI](../project/docs-tooling-and-ci.md).
Other Cargo tools used by CI, such as cargo-hack, cargo-nextest, and cargo-deny, are listed on the pages that use them.

## Changing the baseline

- Toolchain and MSRV bumps within the rules on this page are ordinary pull requests.
- Changing the rules themselves, or moving to a new Rust edition, requires a new ADR that supersedes ADR-0034.
