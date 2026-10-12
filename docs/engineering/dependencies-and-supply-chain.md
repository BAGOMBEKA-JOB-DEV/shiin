<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Dependencies and supply chain

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's dependency and supply-chain rules ([ADR-0038](../adr/0038-supply-chain-policy.md)).

## Admission

Every third-party dependency must be admitted with a reason.
Reviewers weigh:

- How much unsafe code the crate contains, and whether it is necessary.
- Whether its unsafe blocks are documented and tested.
- Its history of soundness advisories in the RustSec database.
- Whether a safe alternative of similar quality exists.

## Declaring dependencies

Every dependency is declared in `[workspace.dependencies]` with a version requirement and `default-features = false`.
Members reference it with `workspace = true` and enable only the features they use.

## Supply chain

- `cargo vet` is enforced in CI from v0.4.
- Dependencies are updated through ordinary pull requests.
- A dependency that gains a soundness advisory is treated as a potential vulnerability.

## Where the policy lives

`deny.toml` at the repository root is the enforced form of this policy, and `ci.yml` runs
`cargo deny check` on every pull request. Two decisions in that file are worth knowing:

- **Two extra licenses are allowed**: `Apache-2.0 WITH LLVM-exception`, which is standard
  for the Rust project's own low-level crates, and `MIT-0`, which is MIT without the
  attribution requirement. Both are permissive.
- **Wildcard versions are refused, except for path dependencies.** Shiin's own crates are
  path dependencies inside this workspace, so they carry no version requirement to review.
  An external wildcard would let an unreviewed upgrade arrive silently.

Duplicate versions warn rather than fail, because they are usually a transitive dependency's
choice rather than ours.

## Read next

- [Workspace and crates](workspace-and-crates.md)
- [Unsafe code policy](unsafe-policy.md)
