<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Release engineering

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's release rules ([ADR-0039](../adr/0039-release-tooling.md)).

## Commits

Commits follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).
Release tooling generates changelogs from them.

## Sign-offs

Every commit carries a Developer Certificate of Origin sign-off, which `git commit -s` adds.

## Versions

Versions before 1.0 may contain breaking changes.
After 1.0, patch, minor, and major versions follow semantic versioning.

## Builds

Binaries are self-contained and fully static on musl targets.

## Artifacts

Signed release binaries with a software bill of materials are produced for every release.

## Read next
- [Versioning and stability](../project/versioning-and-stability.md)
- [Verifying releases](../security/verifying-releases.md)