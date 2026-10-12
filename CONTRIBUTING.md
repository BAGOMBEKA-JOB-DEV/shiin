# Contributing to Shiin

Thank you for considering a contribution.
Shiin is in its specification phase: there is no product code yet, only documentation, specifications, architecture decision records, and a small Rust tool that checks the documentation.

That makes this the cheapest moment to change the design, so review is the most valuable contribution right now.

## What helps most today

- **Review a specification.** Look for ambiguity, contradictions, or requirements that cannot be implemented. Use the specification feedback issue form.
- **Review the [threat model](docs/security/threat-model.md).** Missing threats, wrong assumptions, and unrealistic mitigations are all worth reporting.
- **Describe a use case** that the [current ones](docs/overview/use-cases.md) do not cover.
- **Report a gap in an integration.** If you know a host's hook behavior, the [enforcement levels](docs/security/enforcement-levels.md) page records what each adapter can and cannot see.
- **Improve the documentation.** Anything unclear, wrong, or describing an unbuilt feature as if it worked.

If you believe you have found a way to bypass Shiin's decisions, do not open a public issue.
Follow [SECURITY.md](SECURITY.md) instead.

## Ground rules

- Everyone participating agrees to the [Code of Conduct](CODE_OF_CONDUCT.md).
- Rust is the language for the product and for all tooling, including documentation checks. See [ADR-0002](docs/adr/0002-rust-for-product-and-tooling.md). Please do not add Python or Node tooling.
- Documentation follows the [documentation style guide](docs/project/docs-style-guide.md), and the checks enforce most of it.
- Nothing is implemented, so no page or comment may describe Shiin's behavior as if it already worked.

## Developer Certificate of Origin

Shiin uses the Developer Certificate of Origin instead of a contributor license agreement.
Sign off every commit with `git commit -s`, which appends a line like this:

```text
Signed-off-by: Your Name <your.email@example.com>
```

Use your real name and an address you can be reached at.
Signing off means you certify the following:

```text
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

The canonical text is published at [developercertificate.org](https://developercertificate.org/).

## Workflow

1. Open an issue first for anything beyond a small fix, so the approach can be discussed before you write.
2. Fork the repository and create a branch.
3. Make the change, keeping it scoped to one topic.
4. Write commit messages in [Conventional Commits](https://www.conventionalcommits.org/) style, for example `docs(spec): clarify opaque effect handling`.
5. Run the checks below until they pass.
6. Open a pull request and fill in the template.

## Running the checks

Install Rust with [rustup](https://rustup.rs/); the pinned toolchain is selected automatically.
Then install the documentation tools and run the same commands continuous integration runs:

```console
$ cargo install cargo-binstall
$ cargo binstall --locked mdbook mdbook-mermaid lychee typos-cli rumdl
$ cargo fmt --all --check
$ cargo clippy --workspace --all-targets -- -D warnings
$ cargo xtask docs-check
$ rumdl check .
$ typos
$ lychee --config lychee.toml './*.md' docs
$ mdbook build docs
```

[Documentation tooling and CI](docs/project/docs-tooling-and-ci.md) explains what each check enforces.

## When a decision record or an RFC is needed

- Write an **architecture decision record** when a choice constrains several components, is expensive to reverse, or changes an earlier decision. See the [ADR index](docs/adr/README.md) and copy the [template](docs/adr/template.md).
- Write an **RFC** for changes to an accepted specification, new action-type namespaces, new decision values or stages, policy language features, approval protocol or adapter contract changes, governance changes, or anything that breaks a stable surface. See the [RFC process](docs/rfcs/README.md).
- Bug fixes, refactors, conforming adapters, and documentation corrections need neither.

## Security-sensitive changes

Some paths carry extra review requirements, including the specifications, the security documentation, the future engine and adapter crates, the workflows, and the dependency policy.
[Security-sensitive changes](docs/engineering/security-sensitive-changes.md) lists them and what each pull request must include.
Never weaken fail-closed behavior without an accepted decision record.

## AI-assisted contributions

Using an AI assistant is allowed and, given what Shiin is about, expected.
Two conditions apply.

- Disclose it in the pull request description.
- A human signs off with the Developer Certificate of Origin and takes responsibility for the content, including the accuracy of every factual claim about third-party products.

[AGENTS.md](AGENTS.md) holds the rules an AI assistant should follow in this repository.

## Questions

See [SUPPORT.md](SUPPORT.md).
