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

## Read next
- [Workspace and crates](workspace-and-crates.md)
- [Unsafe code policy](unsafe-policy.md)