<!-- shiin-doc: kind=reference status=draft implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# Glossary

> [!NOTE]
> **Reference: draft.**
> This is the only place Shiin terms are defined.
> Every other page links here instead of redefining a term.

Terms are listed alphabetically.
Each entry gives a definition, what the term is easily confused with, and where its precise rules live.
To propose a new term or change a definition, open a pull request that edits this page
and every page that uses the term.

## Action

An operation an [agent](#agent) attempts that has an effect outside the agent itself,
such as reading a file, running a command, or contacting a host.
The action is Shiin's unit of authorization.

- **Not to be confused with:** a *tool call*, which is a host's name for how an agent invokes a capability.
  One tool call can contain several actions.
- **Rules:** [Action types](spec/action-types.md).

## Action request

The normalized, host-independent description of one proposed [action](#action).
An [adapter](#adapter) builds it and the [engine](#engine) evaluates it.

- **Rules:** [Action request](spec/action-request.md).

## Action type

A dotted identifier that classifies an [action](#action), such as `fs.write` or `shell.exec`.
Policies refer to action types, never to host tool names.

- **Rules:** [Action types](spec/action-types.md).

## Adapter

The component that connects a [host](#host) to Shiin.
An adapter intercepts proposed actions, builds [action requests](#action-request),
and enforces the resulting [decision](#decision) using the host's own protocol.
Adapters never make decisions.

- **Rules:** [Adapter contract](spec/adapter-contract.md).

## Agent

Software that uses a model to propose actions on a person's behalf, such as Claude Code or Codex CLI.
Shiin treats every agent as untrusted.

- **Not to be confused with:** the [host](#host), which is the application the agent runs in.
  For many products the agent and host ship together.

## Agent identity

The identifier of the [agent](#agent) that proposed an action, together with its [assurance level](#assurance-level).

- **Not to be confused with:** approver identity, which identifies the human resolving an [approval request](#approval-request).
- **Rules:** [Identity](spec/identity.md).

## Allow

The [decision](#decision) value meaning the action may proceed.
Only policy [rules](#rule) and [grants](#grant) can produce `allow`.

- **Rules:** [Decision](spec/decision.md), [Evaluation](spec/evaluation.md).

## Approval channel

A route by which [approval requests](#approval-request) reach [approvers](#approver),
such as the command-line approver or a webhook.

- **Rules:** [Approval protocol](spec/approval-protocol.md).

## Approval request

A [pending](#pending) decision waiting for an [approver](#approver) to resolve it.

- **Rules:** [Approval protocol](spec/approval-protocol.md).

## Approver

A human authorized to resolve [approval requests](#approval-request).
An approver *approves* or *denies*; the resulting decision is `allow` or `deny`.

## Assurance level

How strongly an [agent identity](#agent-identity) is established.
The levels, weakest first, are `asserted`, `attested-local`, `attested-os-user`, and `attested-crypto` (reserved for the future).

- **Rules:** [Identity](spec/identity.md).

## Audit event

One record in the [audit log](#audit-log), such as a received request, a decision, or an approval resolution.

- **Rules:** [Audit events](spec/audit-events.md).

## Audit log

An append-only JSON Lines file of [audit events](#audit-event) linked by a [hash chain](#hash-chain).
The audit log is tamper-evident, not tamper-proof.

- **Rules:** [Audit events](spec/audit-events.md).

## Built-in protected resource

A resource that no policy can make allowable for agents,
such as Shiin's own state, keys, and audit log, host hook configuration, and approval commands.

- **Rules:** [Policy model](spec/policy-model.md), [ADR-0016](adr/0016-built-in-protected-resources.md).

## Capability

A coarse permission for an [agent](#agent) to attempt a whole class of [actions](#action) at all,
for example "may run shell commands".
The capability [stage](#stage) is a [gate](#gate).

- **Not to be confused with:** object capabilities (unforgeable tokens), which Shiin does not use.

## Checkpoint

A signed statement of the [audit log](#audit-log)'s head hash at a given position.

- **Rules:** [Audit events](spec/audit-events.md), [Cryptography](security/cryptography.md).

## Classifier

Deterministic code that derives [action types](#action-type) and [effects](#effect) from what a host reports.
Classifiers only ever make decisions stricter:
anything they cannot classify becomes an [opaque effect](#opaque-effect).

- **Rules:** [Action request](spec/action-request.md), [ADR-0018](adr/0018-conservative-action-classification.md).

## Constraint

A machine-checkable limit inside a [task declaration](#task-declaration).
The closed set of constraint kinds is `scope.paths`, `forbid.action_types`, `allow.hosts`, `limits`, `expires_at`, and `require_pending_for`.

- **Rules:** [Intent binding](spec/intent-binding.md).

## Decision

The [engine](#engine)'s answer to an [action request](#action-request):
`allow`, `deny`, or `pending`, with a [reason code](#reason-code).

- **Rules:** [Decision](spec/decision.md).

## Default decision

The decision made when no [rule](#rule) matches.
It is `deny`, can be configured to `pending`, and can never be `allow`.

- **Rules:** [Evaluation](spec/evaluation.md).

## Deny

The [decision](#decision) value meaning the action must not proceed.
A `deny` from any [stage](#stage) wins over every other result, including [grants](#grant).

## Deny-with-ticket

How `pending` is expressed to a [host](#host) that cannot wait for an approver:
the adapter denies the action now and returns an approval ticket.
If the agent retries after an approver approves, the retry consumes a single-use [grant](#grant).

- **Rules:** [Approval protocol](spec/approval-protocol.md), [ADR-0028](adr/0028-deny-with-ticket-for-non-waiting-hosts.md).

## Developer

The person using an agent to do work.
Often the same person as the [operator](#operator) and the [approver](#approver), but the roles are distinct.

## Effect

A concrete consequence of an [action](#action), such as a path written or a host contacted,
recorded with a confidence of `exact`, `inferred`, or `opaque`.

- **Rules:** [Action request](spec/action-request.md).

## Enforcement level

How strongly a Shiin decision is enforced for a given [adapter](#adapter).
At **L1 Cooperative**, the host honors the decision through its hook mechanism, which can be disabled.
At **L2 Mediated**, Shiin sits in the data path, as the MCP proxy does.
At **L3 OS-enforced** (future), the operating system or a sandbox enforces the decision.

- **Rules:** [Enforcement levels](security/enforcement-levels.md), [ADR-0033](adr/0033-enforcement-levels.md).

## Engine

The pure, synchronous component that evaluates an [action request](#action-request)
against a [policy snapshot](#policy-snapshot) and returns a [decision](#decision).
It performs no I/O.

- **Rules:** [Evaluation](spec/evaluation.md), [ADR-0010](adr/0010-pure-synchronous-engine.md).

## Fail-closed

The rule that any failure (an error, a timeout, an unreadable policy) results in `deny`.

- **Rules:** [Security model](security/security-model.md), [ADR-0013](adr/0013-fail-closed-failure-semantics.md).

## Gate

An evaluation [stage](#stage) that can return `deny`, `pending`, or `pass`, but never `allow`.
The identity, capability, intent, and risk stages are gates.

- **Rules:** [Evaluation](spec/evaluation.md).

## Grant

A stored authorization created when an [approver](#approver) resolves an [approval request](#approval-request).

- A **single-use grant** is bound to the digest of one action request.
- A **scoped grant** covers an agent, action types, a resource pattern, and a workspace, and expires.

A grant can satisfy only a grantable `pending` result. It never overrides `deny`.

- **Rules:** [Approval protocol](spec/approval-protocol.md), [ADR-0027](adr/0027-approval-resolutions-and-grants.md).

## Hash chain

The construction in which each [audit event](#audit-event) includes the hash of the event before it,
so removing or altering an event is detectable.

## Hook

A [host](#host) mechanism that runs an external program before or after the agent uses a tool,
and that can accept a decision back.
Hooks are the interception point for [L1](#enforcement-level) adapters.

## Host

The application that runs an [agent](#agent) and exposes an interception mechanism,
such as Claude Code, Codex CLI, Cursor, or an MCP client.

## Intent binding

Checking an [action](#action) against the [constraints](#constraint) of the active [task declaration](#task-declaration).
Natural-language descriptions of intent are recorded but never evaluated.

- **Rules:** [Intent binding](spec/intent-binding.md), [ADR-0020](adr/0020-structured-intent-binding.md).

## Opaque effect

An [effect](#effect) Shiin cannot determine before the action runs,
such as what `python cleanup.py` will do.
Opaque effects lead to `pending` by default.

## Operator

The person who installs and configures Shiin and writes its policy.

## Outcome verification

A reserved concept: checking after an action runs that its real effects matched what was authorized.
It is not yet specified.

- **Not to be confused with:** [receipt verification](#receipt-verification).

## Pending

The [decision](#decision) value meaning the action must wait for an [approver](#approver).
A pending decision that is never resolved expires as `deny`; it never becomes `allow` on its own.

## Policy

The complete set of [rules](#rule), [capabilities](#capability), defaults, and risk configuration
that applies to an evaluation, drawn from every [policy layer](#policy-layer).

- **Rules:** [Policy model](spec/policy-model.md).

## Policy layer

One source of policy.
Layers, in order, are built-in, user, and workspace. An organization layer is reserved for the future.
A workspace layer can only restrict unless the operator trusts it by hash.

- **Rules:** [Policy model](spec/policy-model.md), [ADR-0015](adr/0015-policy-layering-and-trust.md).

## Policy snapshot

The immutable input to one evaluation: compiled policy, grants, active tasks, and counters,
identified by a snapshot hash.

- **Rules:** [Evaluation](spec/evaluation.md).

## Reason code

A stable, machine-readable code that explains a [decision](#decision), such as `deny.policy`.

- **Rules:** [Decision](spec/decision.md).

## Receipt

A signed statement that attests a [decision](#decision) was made,
using a DSSE envelope and an Ed25519 signature.

- **Rules:** [Receipts](spec/receipts.md), [ADR-0031](adr/0031-dsse-ed25519-receipts.md).

## Receipt verification

Checking that a [receipt](#receipt)'s signature and contents are valid.
It proves who signed a statement about a decision, not that the action's real outcome was correct.

## Replay

Re-evaluating recorded [action requests](#action-request) against their recorded [policy snapshots](#policy-snapshot),
or against a different policy to answer "what if".

## Requirement ID

A stable identifier, such as `EVAL-007`, for one normative requirement in a specification.

## Resolution

An [approver](#approver)'s answer to an [approval request](#approval-request): `allow_once`, `allow_always`, or `deny`.

## Risk signal

A discrete, deterministic property of an [action request](#action-request), such as `bulk_delete` or `secret_path`,
that can escalate a decision. Shiin does not use numeric risk scores.

- **Rules:** [Risk signals](spec/risk-signals.md).

## Rule

A [policy](#policy) element that matches action requests and produces `allow`, `deny`, or `pending`.

## Session

One run or conversation of a [host](#host), identified by the host's session ID.

## Shiin

The project. `shiin` is the command-line binary and `shiind` is the daemon.

## Stage

One step of evaluation. The stages, in order, are identity, capability, policy, intent, and risk.
Later stages can only restrict the result of earlier ones.

- **Rules:** [Evaluation](spec/evaluation.md), [ADR-0012](adr/0012-evaluation-stage-order.md).

## Task declaration

A structured, human-confirmed statement of what an agent is working on:
a description, [constraints](#constraint), and an expiry.
Also called a *task*.

- **Rules:** [Intent binding](spec/intent-binding.md).

## Trace

The record of each [stage](#stage)'s result for one decision, produced in explain mode and used by [replay](#replay).

## Workspace

The project directory an agent works in, and the root of the workspace [policy layer](#policy-layer).

## Avoided terms

| Avoid | Use instead | Why |
|---|---|---|
| block, blocked | `deny`, denied | One word for one decision value |
| ask (for Shiin behavior) | `pending` | "Ask" is reserved for hosts' native mechanisms |
| tool call (for Shiin behavior) | action | A tool call can contain several actions |
| permission (for Shiin concepts) | capability, rule | "Permission" is ambiguous with operating-system permissions |
| firewall, guardrail, sandbox (describing Shiin) | authorization layer | Shiin is none of these; see [Vision](overview/vision.md) |
| whitelist, blacklist | allowlist, denylist | Clearer and inclusive |
| user (in specifications) | developer, operator, approver | Specifications need the exact role |
| intent (meaning constraints) | task declaration, constraint | Natural-language intent is never enforced |
| verification (alone) | receipt verification, outcome verification | They prove different things |
