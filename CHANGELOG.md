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
- The ADR-0014 spike in `spikes/cedar/`, which confirmed Cedar as the policy engine by reproducing every specification test vector.
- Evaluation inputs and attended and unattended modes (EVAL-014 to EVAL-017), a required `workspace` object on the action request (AR-008, AR-009), configurable risk thresholds (RISK-007a), the `deny.unattended` and `deny.default` reason codes, and test vector TV-008.

### Changed

- The action request, decision, evaluation, policy model, and adapter contract specifications are `accepted`, and ADR-0014 and ADR-0019 are decided. The only remaining M0 criterion is an outside review of the threat model.
- `cargo xtask docs-check` now rejects a reason code or action type that is not in its registry, including in test vectors.
