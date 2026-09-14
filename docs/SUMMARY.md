# Summary

[Introduction](README.md)

# Overview

- [Vision](overview/vision.md)
- [Use cases](overview/use-cases.md)
- [Landscape](overview/landscape.md)
- [FAQ](overview/faq.md)

# Concepts

- [How Shiin works](concepts/how-shiin-works.md)
- [Action requests](concepts/action-requests.md)
- [Decisions](concepts/decisions.md)
- [Agents and identity](concepts/agents-and-identity.md)
- [Capabilities](concepts/capabilities.md)
- [Policies](concepts/policies.md)
- [Tasks and intent](concepts/tasks-and-intent.md)
- [Risk signals](concepts/risk-signals.md)
- [Approvals](concepts/approvals.md)
- [Adapters](concepts/adapters.md)
- [Audit and evidence](concepts/audit-and-evidence.md)
- [Limitations](concepts/limitations.md)

# Getting started (design intent)

- [Installation](getting-started/installation.md)
- [Quickstart: Claude Code](getting-started/quickstart-claude-code.md)
- [Your first policy](getting-started/first-policy.md)

# Guides (design intent)

- [Writing policies](guides/writing-policies.md)
- [Using Shiin with sandboxes](guides/using-shiin-with-sandboxes.md)
- [Declaring tasks](guides/declaring-tasks.md)
- [Testing policies](guides/testing-policies.md)
- [Handling approvals](guides/handling-approvals.md)
- [Audit, replay, and receipts](guides/audit-replay-and-receipts.md)
- [Integrations]()
  - [Claude Code](guides/integrations/claude-code.md)
  - [Codex CLI](guides/integrations/codex.md)
  - [Cursor](guides/integrations/cursor.md)
  - [MCP proxy](guides/integrations/mcp-proxy.md)
  - [Custom agents](guides/integrations/custom-agents.md)
  - [Aider in a sandbox](guides/integrations/aider-sandboxed.md)

# Reference

- [CLI](reference/cli.md)
- [Configuration](reference/configuration.md)
- [Error codes](reference/error-codes.md)
- [Platform support](reference/platform-support.md)

# Specifications

- [About the specifications](spec/README.md)
- [Action types](spec/action-types.md)
- [Action request](spec/action-request.md)
- [Decision](spec/decision.md)
- [Evaluation](spec/evaluation.md)
- [Identity](spec/identity.md)
- [Policy model](spec/policy-model.md)
- [Policy syntax](spec/policy-syntax.md)
- [Intent binding](spec/intent-binding.md)
- [Risk signals](spec/risk-signals.md)
- [Approval protocol](spec/approval-protocol.md)
- [Audit events](spec/audit-events.md)
- [Receipts](spec/receipts.md)
- [Adapter contract](spec/adapter-contract.md)
- [Local API](spec/local-api.md)
- [Test vectors](spec/test-vectors/README.md)

# Security

- [Threat model](security/threat-model.md)
- [Security model](security/security-model.md)
- [Enforcement levels](security/enforcement-levels.md)
- [Cryptography](security/cryptography.md)
- [Vulnerability disclosure](security/vulnerability-disclosure.md)
- [Hardening](security/hardening.md)
- [Verifying releases](security/verifying-releases.md)

# Architecture

- [Overview](architecture/overview.md)
- [Decision pipeline](architecture/decision-pipeline.md)
- [Deployment models](architecture/deployment-models.md)
- [State and storage](architecture/state-and-storage.md)

# Engineering

- [Workspace and crates](engineering/workspace-and-crates.md)
- [Toolchain and MSRV](engineering/toolchain-and-msrv.md)
- [Coding standards](engineering/coding-standards.md)
- [Error handling](engineering/error-handling.md)
- [Unsafe code policy](engineering/unsafe-policy.md)
- [Async and concurrency](engineering/async-and-concurrency.md)
- [Dependencies and supply chain](engineering/dependencies-and-supply-chain.md)
- [Testing strategy](engineering/testing-strategy.md)
- [Performance budgets](engineering/performance-budgets.md)
- [Continuous integration](engineering/ci.md)
- [Release engineering](engineering/release-engineering.md)
- [Security-sensitive changes](engineering/security-sensitive-changes.md)

# Decisions

- [Architecture decision records](adr/README.md)
  - [0001 Record architecture decisions](adr/0001-record-architecture-decisions.md)
  - [0002 Rust for product and tooling](adr/0002-rust-for-product-and-tooling.md)
  - [0003 Project name: Shiin](adr/0003-project-name-shiin.md)
  - [0004 Apache-2.0 license and DCO](adr/0004-apache-2-license-and-dco.md)
  - [0005 Governance model](adr/0005-governance-model.md)
  - [0006 Vulnerability reporting](adr/0006-vulnerability-reporting.md)
  - [0007 No telemetry](adr/0007-no-telemetry.md)
  - [0008 mdBook documentation](adr/0008-mdbook-documentation.md)
  - [0009 Deterministic enforcement](adr/0009-deterministic-enforcement.md)
  - [0010 Pure synchronous engine](adr/0010-pure-synchronous-engine.md)
  - [0011 Decision values and combining](adr/0011-decision-values-and-combining.md)
  - [0012 Evaluation stage order](adr/0012-evaluation-stage-order.md)
  - [0013 Fail-closed failure semantics](adr/0013-fail-closed-failure-semantics.md)
  - [0014 Cedar policy engine with TOML front-end](adr/0014-cedar-policy-engine-with-toml-front-end.md)
  - [0015 Policy layering and trust](adr/0015-policy-layering-and-trust.md)
  - [0016 Built-in protected resources](adr/0016-built-in-protected-resources.md)
  - [0017 JSON wire format and schemas](adr/0017-json-wire-format-and-schemas.md)
  - [0018 Conservative action classification](adr/0018-conservative-action-classification.md)
  - [0019 Path canonicalization](adr/0019-path-canonicalization.md)
  - [0020 Structured intent binding](adr/0020-structured-intent-binding.md)
  - [0021 Discrete risk signals](adr/0021-discrete-risk-signals.md)
  - [0022 Local-first, daemonless deployment](adr/0022-local-first-daemonless-deployment.md)
  - [0023 JSON-RPC local API](adr/0023-json-rpc-local-api.md)
  - [0024 Tokio, confined](adr/0024-tokio-confined-async-runtime.md)
  - [0025 Adapter roadmap](adr/0025-adapter-roadmap.md)
  - [0026 Adapter extensibility](adr/0026-adapter-extensibility.md)
  - [0027 Approval resolutions and grants](adr/0027-approval-resolutions-and-grants.md)
  - [0028 Deny-with-ticket for hosts that cannot wait](adr/0028-deny-with-ticket-for-non-waiting-hosts.md)
  - [0029 Hash-chained JSON Lines audit log](adr/0029-hash-chained-jsonl-audit-log.md)
  - [0030 Audit durability](adr/0030-audit-durability.md)
  - [0031 DSSE and Ed25519 receipts](adr/0031-dsse-ed25519-receipts.md)
  - [0032 Signing key storage](adr/0032-signing-key-storage.md)
  - [0033 Enforcement levels](adr/0033-enforcement-levels.md)
  - [0034 Edition, MSRV, and toolchain](adr/0034-edition-msrv-and-toolchain.md)
  - [0035 Workspace layout](adr/0035-workspace-layout.md)
  - [0036 Error handling](adr/0036-error-handling.md)
  - [0037 Unsafe code policy](adr/0037-unsafe-code-policy.md)
  - [0038 Supply chain policy](adr/0038-supply-chain-policy.md)
  - [0039 Release tooling](adr/0039-release-tooling.md)
  - [0040 Platform support tiers](adr/0040-platform-support-tiers.md)
  - [0041 Remote approver transport](adr/0041-remote-approver-transport.md)
  - [0042 Language bindings](adr/0042-language-bindings.md)
  - [ADR template](adr/template.md)
- [RFC process](rfcs/README.md)
  - [RFC template](rfcs/0000-template.md)

# Project

- [Roadmap](project/roadmap.md)
- [Versioning and stability](project/versioning-and-stability.md)
- [Privacy and telemetry](project/privacy-and-telemetry.md)
- [Documentation style guide](project/docs-style-guide.md)
- [Documentation tooling and CI](project/docs-tooling-and-ci.md)
- [Maintainer handbook](project/maintainer-handbook.md)

---

[Glossary](glossary.md)
