<!-- shiin-doc: kind=reference status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Platform support

> [!NOTE]
> **Reference: draft.**
> Facts to look up. No implementation exists yet.

## Tiers

| Tier | Platforms | Support |
|---|---|---|
| Tier 1 | Linux x86_64, macOS 13+ (x86_64 and arm64) | Tested in CI. |
| Tier 2 | Windows 10+ (x86_64) | Builds; promoted toward Tier 1 in v0.6. |
| Tier 3 | Other architectures | Not guaranteed to build. |

## Before 1.0

The MSRV is pinned stable minus 2 minor versions.
After 1.0, library crates track latest stable minus 6 minor versions.

## Read next

- [Toolchain and MSRV](../engineering/toolchain-and-msrv.md)
- [Platform support tiers](../adr/0040-platform-support-tiers.md)