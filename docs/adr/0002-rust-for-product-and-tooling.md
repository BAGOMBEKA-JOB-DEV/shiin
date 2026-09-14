<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0002: Rust for product and tooling

> [!NOTE]
> **Architecture decision: accepted.**
> No product code exists yet. The `xtask` documentation tooling is the only Rust crate in the workspace today.

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Workspace and crates](../engineering/workspace-and-crates.md), [Toolchain and MSRV](../engineering/toolchain-and-msrv.md), [Documentation tooling and CI](../project/docs-tooling-and-ci.md), [ADR-0008](0008-mdbook-documentation.md), [ADR-0034](0034-edition-msrv-and-toolchain.md), [ADR-0037](0037-unsafe-code-policy.md), [ADR-0042](0042-language-bindings.md)

## Context and problem statement

Shiin runs on the path of every agent [action](../glossary.md#action): a [host](../glossary.md#host) starts or calls an [adapter](../glossary.md#adapter), waits for its answer, and only then proceeds.
The adapter parses untrusted input from agents and hosts, including shell commands, file paths, and tool payloads.
Developers must be able to install Shiin without first installing a language runtime.
The project also needs repository tooling for documentation checks and releases, and every extra language adds a toolchain that contributors and CI must install, update, and secure.
The question is which language to use for the product, and whether the tooling uses the same one.

## Decision drivers

- Memory safety in code that parses untrusted input.
- Predictable latency on the hook path, with no garbage-collection pauses, within the [performance budgets](../engineering/performance-budgets.md).
- Fast process start-up, because v0.1 runs a new process for each hook invocation ([ADR-0022](0022-local-first-daemonless-deployment.md)).
- Self-contained binaries with no runtime to install.
- Direct, in-process use of the Cedar policy engine, whose reference implementation, [`cedar-policy`](https://github.com/cedar-policy/cedar), is a Rust crate (verified 2026-09-14).
- One toolchain for contributors, CI, and the supply-chain controls in [ADR-0038](0038-supply-chain-policy.md).

## Considered options

1. Rust for the product and all tooling
2. Rust for the product, with Python or Node.js scripts for tooling
3. Go for the product and tooling
4. TypeScript on Node.js for the product and tooling

## Decision outcome

Chosen option: "Rust for the product and all tooling", because it meets the safety, latency, and distribution drivers and lets the project run a single toolchain.

- Every product component is a Rust crate in one Cargo workspace, as listed in [Workspace and crates](../engineering/workspace-and-crates.md).
- Repository tooling is Rust.
  Checks run through `cargo xtask`, and the documentation tools (`mdbook`, `mdbook-mermaid`, `lychee`, `typos`, and `rumdl`) are installed with `cargo install --locked` ([ADR-0008](0008-mdbook-documentation.md)).
- Building, testing, documenting, and releasing Shiin require no Python and no Node.js.
  CI workflow files are configuration, not a second tooling language.
- Binaries are self-contained, and fully static on musl targets.
- The edition, MSRV, and toolchain pinning are decided in [ADR-0034](0034-edition-msrv-and-toolchain.md), and the rules for `unsafe` code in [ADR-0037](0037-unsafe-code-policy.md).
- Agents written in other languages integrate through the JSON wire format ([ADR-0017](0017-json-wire-format-and-schemas.md)) and, later, language bindings ([ADR-0042](0042-language-bindings.md)).

### Consequences

- Good, because memory-safety bugs are excluded from the parsers that handle untrusted input, outside the single crate allowed to contain `unsafe` code.
- Good, because the policy engine is linked in-process rather than called across a language boundary.
- Good, because contributors install one toolchain and CI caches one.
- Good, because other agent-authorization projects written in Rust, such as [SECBLOK/belay](https://github.com/SECBLOK/belay), [eqtylab/cupcake](https://github.com/eqtylab/cupcake), and [agentgateway](https://github.com/agentgateway/agentgateway), show the ecosystem suits this domain (verified 2026-09-14).
- Bad, because compile times are long, for CI and for first-time contributors.
- Bad, because fewer contributors know Rust than Python, TypeScript, or Go.
- Bad, because small repository tasks take more code as `xtask` commands than as scripts.
- Bad, because agent builders in other languages wait for bindings instead of getting a native library from the start.

### Confirmation

- The documentation workflow described in [Documentation tooling and CI](../project/docs-tooling-and-ci.md) installs only the Rust toolchain and Cargo-installed tools (planned: M0).
- `cargo xtask` is the single entry point for repository checks.
- Review rule: a pull request that adds a Python, Node.js, or other non-Rust dependency to building, testing, documenting, or releasing Shiin is not merged without an RFC and a superseding ADR.

## Pros and cons of the options

### Rust for everything

- Good, because it satisfies every decision driver.
- Bad, because of the compile-time and contributor-pool costs listed above.

### Rust product with scripting-language tooling

- Good, because scripting languages are quick for small repository tasks.
- Bad, because contributors and CI must install and secure a second toolchain and package ecosystem.
- Bad, because the Rust supply-chain controls do not cover that ecosystem.

### Go

- Good, because Go is memory safe, compiles quickly, and produces self-contained binaries.
- Bad, because garbage collection adds latency variance on the hook path.
- Bad, because Cedar would be used through a separate implementation or a foreign-function boundary instead of the reference crate.

### TypeScript on Node.js

- Good, because TypeScript is widely known, and some peers, such as [Adirdabush1/cerberus](https://github.com/Adirdabush1/cerberus), use it (verified 2026-09-14).
- Bad, because users must install a Node.js runtime, and every hook invocation pays its start-up cost.
- Bad, because a security component would depend on a second large package ecosystem.

## More information

Revisit this decision only if a required capability has no viable Rust implementation.
Demand for other languages is handled by [ADR-0042](0042-language-bindings.md), not by changing the implementation language.
