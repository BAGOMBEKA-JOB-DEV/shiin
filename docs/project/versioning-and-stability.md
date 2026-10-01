<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Versioning and stability

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's versioning and stability rules.

## Versions before 1.0

Versions before 1.0 may contain breaking changes.
A breaking change adds `!` after the scope in the commit subject and a `BREAKING CHANGE:` footer.

## Versions after 1.0

- **Patch** (x.y.Z): Backward-compatible fixes only.
- **Minor** (x.Y.z): Backward-compatible features. May raise the MSRV.
- **Major** (X.y.z): Breaking changes.

## Specifications

The v1 specifications and the policy language are frozen under the versioning policy.
Before 1.0, specifications are versioned as `v1-draft.N` and may change.

## Library crates

`shiin-schema` and `shiin-client` are the only crates intended to become stable public APIs.
After 1.0 they follow the guarantees in this page.

## Read next

- [Roadmap](roadmap.md)
- [Workspace and crates](../engineering/workspace-and-crates.md)