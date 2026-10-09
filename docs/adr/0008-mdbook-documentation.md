<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0008: mdBook documentation

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [ADR-0002](0002-rust-for-product-and-tooling.md), [ADR-0001](0001-record-architecture-decisions.md), [Documentation tooling and CI](../project/docs-tooling-and-ci.md)

## Context and problem statement

Shiin's documentation is written before its implementation, and it must render both on GitHub and as a dedicated site.
The project needed a documentation generator that reads plain Markdown, supports cross-references and diagrams, and produces a searchable site without requiring a Node.js toolchain.

## Decision drivers

- Renders the same Markdown on GitHub and as a site.
- Supports Mermaid diagrams (used throughout the design).
- No JavaScript runtime required to build.
- Integrates into the Rust toolchain that the project already uses.

## Considered options

1. mdBook with `mdbook-mermaid`, vendored JavaScript.
2. A static-site generator written in Python or Node.js.
3. Plain GitHub Pages with no build step.

## Decision outcome

Chosen option: "mdBook with mdbook-mermaid", because it is written in Rust, renders GitHub-compatible Markdown, and has a Mermaid preprocessor.

- The source lives in `docs/` with `SUMMARY.md` as the table of contents.
- [ADR-0002](0002-rust-for-product-and-tooling.md) requires the documentation toolchain to be installable with `cargo install --locked`.
- Mermaid renderer files are vendored (`docs/mermaid.min.js`, `docs/mermaid-init.js`) so the built site works offline.
- The built site output goes to `target/book/`.

### Consequences

- Good, because contributors install only Rust tooling.
- Good, because the same Markdown files render correctly on GitHub.
- Bad, because mdBook's rendering of admonitions depends on GitHub-flavored Markdown extensions, which mdBook supports but not every Markdown viewer does.

### Confirmation

- `docs/book.toml` configures the build.
- `mdbook build docs` succeeds, as enforced by [Documentation tooling and CI](../project/docs-tooling-and-ci.md).

## Pros and cons of the options

### mdBook

- Good, because it is Rust-native and matches the toolchain decision.
- Good, because it supports Mermaid via `mdbook-mermaid`.
- Bad, because it requires vendored JavaScript for Mermaid rendering.

### Python or Node.js generator

- Good, because those ecosystems have many themes.
- Bad, because they require a second toolchain.

### Plain GitHub Pages

- Good, because it requires no build step.
- Bad, because it cannot render Mermaid diagrams without JavaScript, and it offers no cross-reference validation.

## More information

The [documentation style guide](../project/docs-style-guide.md) defines the Markdown conventions the book relies on. Upgrading `mdbook-mermaid` requires rerunning `mdbook-mermaid install docs` to regenerate vendored files.
