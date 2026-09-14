<!-- shiin-doc: kind=reference status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# Architecture decision records

> [!NOTE]
> **Reference: accepted.**
> The index of every architecture decision in the project.

An architecture decision record (ADR) captures one significant decision:
the context, the options considered, the choice, and its consequences.
ADRs are never deleted. A decision that changes is *superseded* by a new ADR that links back to it.

## When to write an ADR

Write an ADR when a decision:

- constrains how several components or future contributors work,
- is expensive to reverse,
- chooses between credible alternatives, or
- changes a previous ADR.

Changes to accepted specifications, new action-type namespaces, and policy-language features
go through the [RFC process](../rfcs/README.md), and the RFC produces the ADR.

## Statuses

| Status | Meaning |
|---|---|
| `proposed` | A recommendation is written but not yet ratified, or its confirmation step is still pending |
| `accepted` | The decision is in effect |
| `deferred` | The question is recognized but intentionally postponed to a named milestone |
| `superseded` | Replaced by a later ADR, which is linked |
| `deprecated` | No longer relevant, with no replacement |

## Process

1. Copy the [template](template.md) to `NNNN-short-title.md` using the next free number.
2. Open a pull request with status `proposed`, and add the record to the index below.
3. A maintainer accepts the decision by changing its status when the pull request merges,
   following [GOVERNANCE.md](https://github.com/BAGOMBEKA-JOB-DEV/shiin/blob/main/GOVERNANCE.md).
4. To change a decision, write a new ADR that supersedes the old one, and update both records.

## Index

### Project

| ADR | Title | Status |
|---|---|---|
| 0001 | [Record architecture decisions](0001-record-architecture-decisions.md) | accepted |
| 0002 | [Rust for product and tooling](0002-rust-for-product-and-tooling.md) | accepted |
| 0003 | [Project name: Shiin](0003-project-name-shiin.md) | accepted |
| 0004 | [Apache-2.0 license and DCO](0004-apache-2-license-and-dco.md) | accepted |
| 0005 | [Governance model](0005-governance-model.md) | accepted |
| 0006 | [Vulnerability reporting](0006-vulnerability-reporting.md) | accepted |
| 0007 | [No telemetry](0007-no-telemetry.md) | accepted |
| 0008 | [mdBook documentation](0008-mdbook-documentation.md) | accepted |

### Engine semantics

| ADR | Title | Status |
|---|---|---|
| 0009 | [Deterministic enforcement](0009-deterministic-enforcement.md) | accepted |
| 0010 | [Pure synchronous engine](0010-pure-synchronous-engine.md) | accepted |
| 0011 | [Decision values and combining](0011-decision-values-and-combining.md) | accepted |
| 0012 | [Evaluation stage order](0012-evaluation-stage-order.md) | accepted |
| 0013 | [Fail-closed failure semantics](0013-fail-closed-failure-semantics.md) | accepted |
| 0014 | [Cedar policy engine with TOML front-end](0014-cedar-policy-engine-with-toml-front-end.md) | proposed |

### Policy, classification, intent, and risk

| ADR | Title | Status |
|---|---|---|
| 0015 | [Policy layering and trust](0015-policy-layering-and-trust.md) | accepted |
| 0016 | [Built-in protected resources](0016-built-in-protected-resources.md) | accepted |
| 0017 | [JSON wire format and schemas](0017-json-wire-format-and-schemas.md) | accepted |
| 0018 | [Conservative action classification](0018-conservative-action-classification.md) | accepted |
| 0019 | [Path canonicalization](0019-path-canonicalization.md) | proposed |
| 0020 | [Structured intent binding](0020-structured-intent-binding.md) | accepted |
| 0021 | [Discrete risk signals](0021-discrete-risk-signals.md) | accepted |

### Deployment, API, adapters, and approvals

| ADR | Title | Status |
|---|---|---|
| 0022 | [Local-first, daemonless deployment](0022-local-first-daemonless-deployment.md) | accepted |
| 0023 | [JSON-RPC local API](0023-json-rpc-local-api.md) | accepted |
| 0024 | [Tokio, confined](0024-tokio-confined-async-runtime.md) | accepted |
| 0025 | [Adapter roadmap](0025-adapter-roadmap.md) | accepted |
| 0026 | [Adapter extensibility](0026-adapter-extensibility.md) | accepted |
| 0027 | [Approval resolutions and grants](0027-approval-resolutions-and-grants.md) | accepted |
| 0028 | [Deny-with-ticket for hosts that cannot wait](0028-deny-with-ticket-for-non-waiting-hosts.md) | accepted |

### Evidence

| ADR | Title | Status |
|---|---|---|
| 0029 | [Hash-chained JSON Lines audit log](0029-hash-chained-jsonl-audit-log.md) | accepted |
| 0030 | [Audit durability](0030-audit-durability.md) | accepted |
| 0031 | [DSSE and Ed25519 receipts](0031-dsse-ed25519-receipts.md) | accepted |
| 0032 | [Signing key storage](0032-signing-key-storage.md) | accepted |
| 0033 | [Enforcement levels](0033-enforcement-levels.md) | accepted |

### Rust engineering

| ADR | Title | Status |
|---|---|---|
| 0034 | [Edition, MSRV, and toolchain](0034-edition-msrv-and-toolchain.md) | accepted |
| 0035 | [Workspace layout](0035-workspace-layout.md) | accepted |
| 0036 | [Error handling](0036-error-handling.md) | accepted |
| 0037 | [Unsafe code policy](0037-unsafe-code-policy.md) | accepted |
| 0038 | [Supply chain policy](0038-supply-chain-policy.md) | accepted |
| 0039 | [Release tooling](0039-release-tooling.md) | accepted |
| 0040 | [Platform support tiers](0040-platform-support-tiers.md) | accepted |

### Deferred

| ADR | Title | Status |
|---|---|---|
| 0041 | [Remote approver transport](0041-remote-approver-transport.md) | deferred to v0.6 |
| 0042 | [Language bindings](0042-language-bindings.md) | deferred until after 1.0 |
