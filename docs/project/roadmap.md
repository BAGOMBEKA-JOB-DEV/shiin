<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# Roadmap

> [!NOTE]
> **Project policy: draft.**
> This roadmap is the project's plan of record. It changes through pull requests, and larger changes go through the [RFC process](../rfcs/README.md).

## How to read this roadmap

- Milestones are defined by capability, not by date.
  A milestone is finished when its exit criteria are met.
- Each milestone lists what is out of scope, so that "not yet" is an explicit decision.
- Shipping a milestone includes updating the documentation:
  every page for that milestone changes its banner from "Design intent: not implemented" to the implemented state.
- Versions before 1.0 may contain breaking changes, as described in [Versioning and stability](versioning-and-stability.md).

## Overview

| Milestone | Theme | Headline capability |
|---|---|---|
| [M0](#m0-specification) | Specification | Specs, threat model, test vectors, and the policy-engine spike |
| [v0.1](#v01-gate) | Gate | Claude Code adapter with policy, basic risk signals, and an audit log |
| [v0.2](#v02-tasks) | Tasks | Task declarations and intent binding; Codex CLI and Cursor adapters |
| [v0.3](#v03-approve) | Approve | Daemon, approval protocol, and grants |
| [v0.4](#v04-evidence) | Evidence | Signed receipts, verification, and replay |
| [v0.5](#v05-mcp-and-sdk) | MCP and SDK | MCP proxy, local API, and the Rust client library |
| [v0.6](#v06-reach) | Reach | Remote approvals, streamable-HTTP MCP, shell wrapper, Windows |
| [v1.0](#v10-stable) | Stable | Frozen specifications, audited, conformance suite |
| [After 1.0](#after-10) | Teams | Team policy server, more approver apps, language bindings |

## M0: Specification

**Goal.** Settle the design before writing product code.

**Scope.**

- The documentation set in `docs/`, including every specification listed in [About the specifications](../spec/README.md).
- JSON Schemas and test vectors that encode the [use cases](../overview/use-cases.md).
- The Cedar spike required by [ADR-0014](../adr/0014-cedar-policy-engine-with-toml-front-end.md).

**Exit criteria.**

- The action request, decision, evaluation, policy model, and adapter contract specifications are `accepted`.
- The threat model has been reviewed by at least one person outside the project.
- Every golden use case has test vectors.
- ADR-0014 is `accepted`, either confirming Cedar or recording the fallback.

## v0.1: Gate

**Goal.** A developer using Claude Code on Linux or macOS can enforce a policy and see a record of every decision.

**Scope.**

- Claude Code hook adapter, running without a daemon.
- Identity at the `asserted` level, capability, policy, and the `secret_path`, `outside_workspace`, `bulk_delete`, and `opaque_command` risk signals.
- Built-in protected resources.
- Commands: `shiin init`, `shiin check`, `shiin explain`, `shiin policy validate`, `shiin hook claude-code`, `shiin log`, and `shiin doctor`.
- Hash-chained audit log.
- `pending` expressed through Claude Code's native permission prompt. This is an interim step, not the approval protocol.
- Signed release binaries with a software bill of materials.

**Out of scope.** Daemon, approval protocol, grants, tasks, receipts, Windows.

**Exit criteria.**

- All v0.1 test vectors pass.
- The performance budgets for the daemonless hook path are met on Tier 1 platforms.
- A fault-injection test shows that crashes and timeouts in the hook produce `deny`.

## v0.2: Tasks

**Goal.** Actions can be bound to the task the developer gave, and the same policy works across three hosts.

**Scope.**

- Task declarations with the closed constraint set, marked experimental.
- Counters for task limits.
- Codex CLI and Cursor hook adapters.
- `shiin task` and `shiin policy test`.
- Unattended mode for CI.

**Out of scope.** Approval protocol, SQL classification.

**Exit criteria.**

- Intent-binding test vectors pass for all three hosts.
- The intent-binding evaluation plan in [Intent binding](../spec/intent-binding.md) has produced its first measurements.

## v0.3: Approve

**Goal.** `pending` decisions go through a real approval protocol instead of host prompts.

**Scope.**

- `shiind`, the per-user daemon.
- Approval protocol v1 with the command-line approver.
- Single-use and scoped grants, and revocation.
- Deny-with-ticket for hosts that cannot wait.

**Out of scope.** Remote approvers, desktop or editor approvers.

**Exit criteria.**

- Approval state-machine property tests pass.
- Self-approval through an intercepted channel is shown to be impossible by test.

## v0.4: Evidence

**Goal.** Decisions can be proven and re-examined.

**Scope.**

- Signed checkpoints and receipts.
- `shiin verify` and `shiin replay`, including what-if replay against a different policy.
- Key management and rotation.
- Redaction of sensitive values in audit events.
- cargo-vet enforced in CI.

**Exit criteria.**

- Receipts verify offline on every Tier 1 platform.
- Replay reproduces every decision in the conformance corpus.

## v0.5: MCP and SDK

**Goal.** Shiin mediates MCP traffic and can be embedded in custom agents.

**Scope.**

- MCP proxy for spec revision 2026-07-28, stdio transport first.
- Local API over the daemon.
- `shiin-client`, the Rust client library.
- Peer-credential identity (`attested-local`).
- SQL and migration classifiers.

**Exit criteria.**

- The MCP proxy passes conformance tests against reference MCP servers.
- A custom agent built with `shiin-client` passes the adapter conformance suite.

## v0.6: Reach

**Goal.** Shiin works in more environments.

**Scope.**

- Remote approvers through signed webhooks ([ADR-0041](../adr/0041-remote-approver-transport.md) must be decided first).
- Streamable-HTTP MCP transport.
- Shell wrapper for hosts without hooks.
- Windows promoted toward Tier 1.
- Additional hosts, each with its hook behavior verified and documented.

## v1.0: Stable

**Goal.** Shiin is dependable enough to build on.

**Exit criteria.**

- The v1 specifications and the policy language are frozen.
- An independent security audit is complete, and its report is published.
- The conformance suite is published.
- All success criteria in the [vision](../overview/vision.md#success-criteria-for-10) are met.

## After 1.0

- Team policy server with signed policy bundles and central evidence.
- Desktop and editor approver applications.
- Language bindings ([ADR-0042](../adr/0042-language-bindings.md)).

## Not planned

These are deliberately not on the roadmap. Proposing any of them requires an RFC.

- Machine-learning or language-model decisions.
- Telemetry or any phone-home behavior.
- Dynamic-library plugins for adapters ([ADR-0026](../adr/0026-adapter-extensibility.md)).
- Compliance certifications.

## Changing the roadmap

Small corrections are ordinary pull requests.
Adding, removing, or reordering milestone scope requires an [RFC](../rfcs/README.md).
