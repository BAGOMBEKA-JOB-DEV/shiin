<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Continuous integration

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's CI rules.

## Workflows

| Workflow | Triggers | What it runs |
|---|---|---|
| `docs` | PRs and pushes to `main` | `cargo fmt --check`, `cargo clippy`, `cargo xtask docs-check`, `rumdl`, `typos`, `lychee`, `mdbook build`. |
| `ci` | PRs and pushes to `main` | `cargo check`, `cargo test`, `cargo hack check --rust-version`, `cargo nextest`. |

## Permissions

The `docs` workflow has read-only `contents` permission and does not persist checkout credentials.

## Third-party actions

Third-party actions are pinned by commit SHA, with the release tag in a comment.

## Nightly jobs

Nightly jobs use a dated nightly named in the workflow, updated by pull request.

## Read next
- [Documentation tooling and CI](../project/docs-tooling-and-ci.md)
- [Testing strategy](testing-strategy.md)