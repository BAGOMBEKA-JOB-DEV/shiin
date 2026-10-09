<!-- shiin-doc: kind=adr status=proposed implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0014: Cedar policy engine with TOML front-end

> [!NOTE]
> **Architecture decision: proposed.**
> Awaiting a spike to confirm Cedar integration or record the fallback.

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Policy model](../spec/policy-model.md), [Policy syntax](../spec/policy-syntax.md), [Vision](../overview/vision.md#what-makes-shiin-different), [Landscape](../overview/landscape.md), [ADR-0002](0002-rust-for-product-and-tooling.md)

## Context and problem statement

Shiin needs an authorization engine that evaluates policy rules against action requests.
Operators need a syntax that is easy to read and write, while the engine needs an underlying formal model that is testable, explainable, and reasoned about.
Two layers are needed: a human-facing syntax and a machine-verifiable evaluation model.

## Decision drivers

- Policy must be analyzable: testable, explainable, and formally reasoned about.
- The front-end syntax must be approachable for operators writing their first policy.
- The engine implementation must be in Rust (see [ADR-0002](0002-rust-for-product-and-tooling.md)).
- Cedar's reference implementation, `cedar-policy`, is a Rust crate.

## Considered options

1. Compile TOML policy to Cedar, with Cedar as the evaluation engine.
2. Write a native Rust policy evaluator behind a TOML front-end.

## Decision outcome

Chosen option: "Compile TOML policy to Cedar", because Cedar is formally verified and provides explainability and testability that a custom evaluator would need to reimplement.

- Operators write policy in TOML, as shown in [How Shiin works](../concepts/how-shiin-works.md).
- The `shiin-policy` crate compiles TOML to Cedar at policy load time.
- Cedar's `cedar-policy` Rust crate performs evaluation.
- Advanced users can bypass the TOML front-end and write Cedar directly.
- This decision is proposed and awaits the M0 spike. The spike either confirms Cedar or records a superseding ADR with the fallback.

### Consequences

- Good, because policy can be tested, explained, and reasoned about using Cedar's tooling.
- Good, because a formally verified engine reduces the risk of authorization bugs.
- Bad, because it adds the `cedar-policy` dependency, which increases compile time and supply-chain surface.
- Bad, because if the spike finds Cedar unsuitable, the TOML-to-Cedar compilation work is thrown away.

### Confirmation

- The M0 exit criterion requires [ADR-0014](0014-cedar-policy-engine-with-toml-front-end.md) to be `accepted`, either confirming Cedar or recording the fallback.
- The [policy syntax specification](../spec/policy-syntax.md) is stable regardless of the evaluation backend.
- The use-case checklist in this ADR must pass as a test vector set.

## Pros and cons of the options

### Compile TOML to Cedar

- Good, because Cedar is formally verified and analyzable.
- Good, because the Cedar ecosystem provides explainability and testing tools.
- Bad, because it depends on a third-party crate.

### Native Rust evaluator

- Good, because it has no external policy-engine dependency.
- Bad, because it is not formally verified and must reimplement Cedar's analysis features.

## More information

The spike is scoped in the [roadmap](../project/roadmap.md#m0-specification). The landscape page compares Cedar and other policy engines. If this ADR is superseded, the replacement records the fallback evaluator's design.
