# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Versioning and stability](docs/project/versioning-and-stability.md).
Shiin has not been released yet, so every entry below is unreleased.

## [Unreleased]

### Added

- The documentation set under `docs/`, including the vision, concepts, architecture, and project policies.
- Normative specifications for action types, action requests, decisions, evaluation, identity, the policy model and syntax, intent binding, risk signals, the approval protocol, audit events, receipts, the adapter contract, and the local API, with JSON Schemas and test vectors.
- Architecture decision records 0001 to 0042.
- The security documentation: threat model, security model, enforcement levels, cryptography, hardening, release verification, and the disclosure policy.
- Rust engineering standards covering the workspace, toolchain and minimum supported Rust version, coding standards, error handling, unsafe code, concurrency, dependencies, testing, performance budgets, continuous integration, and releases.
- `cargo xtask docs-check`, the Rust tool that enforces the documentation rules, together with the mdBook build and the link, spelling, and Markdown checks.
- Community files: contribution guide, code of conduct, security policy, support, governance, and issue and pull request templates.
- The first product crates: `shiin-schema`, `shiin-classify`, `shiin-policy`, `shiin-engine`, `shiin-audit`, `shiin-hooks`, and the `shiin` command-line interface, with a conformance test that runs the whole specification corpus.
- Evaluation inputs and attended and unattended modes (EVAL-014 to EVAL-017), a required `workspace` object on the action request (AR-008, AR-009), configurable risk thresholds (RISK-007a), the `deny.unattended` and `deny.default` reason codes, and test vector TV-008.

### Changed

- The action request, decision, evaluation, policy model, and adapter contract specifications are `accepted`, and ADR-0014 and ADR-0019 are decided. The only remaining M0 criterion is an outside review of the threat model.
- `cargo xtask docs-check` now rejects a reason code or action type that is not in its registry, including in test vectors.
- Effects may name their own action type, so a rule written for `fs.read` also covers a read performed by a shell command (AR-010, AR-011, TV-009). Found while implementing use case UC-02.
- The ADR-0014 spike was retired once `shiin-policy` and `shiin-engine` covered the same corpus, as its disposition note required.
- Stable error codes are emitted by the crates and documented, and `cargo xtask docs-check` now rejects a code documented as emitted that no crate emits, a code marked reserved that a crate does emit, and a code in the source that is documented nowhere.
- Fault-injection tests cover the paths that would otherwise become an allow: an unparsable policy, an unknown policy key, an unwritable audit log, an empty payload, a tampered log, and a decision that arrives too close to a host timeout.
- The getting-started guides, the policy guide, and the CLI, configuration, and error-code references now describe what a source build does, with each command and setting marked as working today or planned.
- Release engineering: `dist-workspace.toml` for the six supported targets with shell and PowerShell installers, the dist-generated `release.yml` with every action pinned by commit, build provenance attestations, and `sbom.yml`, which produces a CycloneDX bill of materials and attaches it to a release.
- `deny.toml`, which enforces the documented dependency policy, and a `ci.yml` workflow running tests per platform tier, the minimum supported Rust version, `cargo deny`, and a check that no action is pinned by tag or branch.
- `shiin version`, so a bug report can name the build.
- Task declarations, the v0.2 differentiator: `shiin task start | show | end`, with scope, forbidden action types, allowed hosts, counting limits, ask-for, and expiry. A refused action never spends a task's budget, and an expired task stops constraining.
- Codex CLI and Cursor hook adapters, each speaking its host's own protocol, with Cursor's `failClosed` requirement and Codex's unverified payload field names documented rather than hidden.
- `shiin policy test`, which runs an operator's own cases through the same runner as the project's conformance corpus.
- Four more conformance vectors (TV-010 to TV-013) covering task limits, expiry, ask-for, and host constraints.
