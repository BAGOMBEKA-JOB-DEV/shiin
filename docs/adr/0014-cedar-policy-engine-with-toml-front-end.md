<!-- shiin-doc: kind=adr status=proposed implementation=n/a milestone=m0 reviewed=2026-10-01 -->

# ADR-0014: Cedar policy engine with TOML front-end

> [!NOTE]
> **Architecture decision: proposed.**
> A recommendation is written but not yet ratified, or its confirmation step is still pending.

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Policy model](../spec/policy-model.md), [Policy syntax](../spec/policy-syntax.md), [ADR-0002](0002-rust-for-product-and-tooling.md)

## Context and problem statement

Policy must be analyzable: testable, explainable, and reason about.
Cedar is a CNCF Sandbox authorization language formally verified in Lean, with a Rust crate.
The question is whether to use Cedar behind a TOML front-end.

## Considered options

1. Cedar with a TOML front-end
2. A native Rust evaluator behind the same policy model
3. Rego compiled to WebAssembly

## Decision outcome

Chosen option: Cedar with a TOML front-end, proposed.
Operators write TOML policy, which Shiin compiles deterministically to Cedar.
Advanced users can write Cedar directly.
This choice is proposed and awaits a spike.
The fallback is a native Rust evaluator behind the same policy model.

## Consequences
- Policies can be tested, explained, and reasoned about.
- The engine links Cedar in-process.
- A spike is required to confirm.

## Confirmation
- A Cedar spike is required in M0.
- The spike result confirms or records the fallback.
