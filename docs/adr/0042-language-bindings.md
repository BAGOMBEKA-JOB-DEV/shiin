<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=post-1.0 reviewed=2026-09-14 -->

# ADR-0042: Language bindings

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [ADR-0002](0002-rust-for-product-and-tooling.md), [ADR-0017](0017-json-wire-format-and-schemas.md), [ADR-0023](0023-json-rpc-local-api.md), [Workspace and crates](../engineering/workspace-and-crates.md)

## Context and problem statement

The Rust client library (`shiin-client`) is the primary way to embed Shiin in custom agents and tools.
But many agent developers use Python, TypeScript, Java, or Go.
After 1.0, the project needs a path for non-Rust integrators to use Shiin without writing Rust.

## Decision drivers

- The stable public API consists of `shiin-schema` and `shiin-client` only.
- Bindings must not pull in the full engine or Cedar crate.
- Bindings should be generated or maintained with minimal manual effort.

## Considered options

1. Ship `shiin-client` as the Rust native library, then provide FFI bindings for other languages after 1.0.
2. Provide official bindings for Python and TypeScript from the start.
3. Rely on third parties to write unofficial bindings using the local API.

## Decision outcome

Chosen option: "Rust native first, FFI or language-wrapper bindings after 1.0", because a single stable Rust API ensures consistency, and bindings can be built on top of it.

- The Rust client library ships in v0.5 ([ADR-0002](0002-rust-for-product-and-tooling.md) rationale: one toolchain).
- Non-Rust bindings are added after 1.0 when the Rust API is stable.
- Bindings use the local API ([ADR-0023](0023-json-rpc-local-api.md)) or FFI over `shiin-client`.
- The JSON wire format ([ADR-0017](0017-json-wire-format-and-schemas.md)) enables any language to interoperate even before official bindings.

### Consequences

- Good, because the focus is on a correct, stable Rust API first.
- Good, because the JSON wire format allows unofficial integrations from day one.
- Bad, because non-Rust developers must use the raw JSON RPC or write their own adapter until bindings ship.

### Confirmation

- The [workspace and crates page](../engineering/workspace-and-crates.md) lists `shiin-client` as the only stable public Rust API (besides `shiin-schema`).
- The [local API specification](../spec/local-api.md) is designed to be implementable from any language.

## Pros and cons of the options

### Rust native first, bindings after 1.0

- Good, because the API is correct and stable before binding work.
- Bad, because non-Rust teams wait for bindings.

### Python and TypeScript from the start

- Good, because it captures those markets early.
- Bad, because it splits the team across multiple language ecosystems and delays the Rust API.

### Third-party bindings only

- Good, because the core team is not responsible.
- Bad, because quality and maintenance vary.

## More information

The [vision](../overview/vision.md#who-shiin-is-for) states that after 1.0, teams need shared policy and language bindings. The [local API specification](../spec/local-api.md) provides the contract that any language binding can implement.
