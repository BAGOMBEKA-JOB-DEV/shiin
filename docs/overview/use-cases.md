<!-- shiin-doc: kind=explanation status=draft implementation=none milestone=n/a reviewed=2026-09-14 -->

# Use cases

> [!NOTE]
> **Design document: draft.**
> Describes the intended design. No implementation exists yet.

This page describes the situations Shiin is designed for.
Each scenario has a stable ID (`UC-01` and so on).
The scenarios form the *golden set*: the [test vectors](../spec/test-vectors/README.md) encode them,
and the policy language must be able to express every one of them
(see the policy-model checklist in [ADR-0014](../adr/0014-cedar-policy-engine-with-toml-front-end.md)).

## Personas

| Persona | Situation | What they need from Shiin |
|---|---|---|
| **Solo developer** | Uses two or three agents on a laptop, often with broad permissions to keep work flowing | Protection from costly mistakes without constant prompts |
| **Security-minded lead** | Responsible for a team adopting agents, needs to explain the risk to others | Clear, reviewable policy and evidence of what agents did |
| **Platform engineer** | Runs agents in CI or other unattended environments | Deterministic behavior with no human in the loop |
| **Agent builder** | Maintains an agent, host, or MCP server | A standard way to ask whether an action is authorized |

## Scenarios

### UC-01: One policy across several agents

**Situation.** A developer uses Claude Code for feature work and Codex CLI for reviews.
Each host has its own permission configuration, and they disagree.

**Today's gap.** Two formats, two sets of rules, no shared view.

**With Shiin.** Both hosts run Shiin adapters that read the same `.shiin/policy.toml`.
Because policy refers to [action types](../spec/action-types.md) rather than tool names,
a rule such as "agents may not read `.env` files" applies to both.

**Needs.** Claude Code and Codex adapters, policy model, action types.
**Milestone.** v0.2.

### UC-02: Keep secrets out of reach

**Situation.** The workspace contains `.env` files, and the home directory contains SSH keys and cloud credentials.

**Today's gap.** Host permission rules can deny reads through the host's file tools,
but the same secrets can be read through a shell command such as `cat .env`.

**With Shiin.** The classifier maps both the file-read tool and `cat .env` to an `fs.read` effect on the same canonical path,
so a single deny rule covers both.
Reads through code the agent runs (`python -c 'open(".env")'`) are opaque and produce `pending`, not `allow`.

**Needs.** Path canonicalization, shell classifier, `secret_path` risk signal.
**Milestone.** v0.1.

### UC-03: Guard against bulk deletion

**Situation.** An agent "cleans up" by deleting many files at once.

**Today's gap.** Hosts either ask about every command, which leads to approval fatigue,
or allow commands broadly.

**With Shiin.** Deletions within normal limits proceed under policy.
When a single action would delete more files than the configured threshold,
the `bulk_delete` risk signal raises the decision to `pending`.

**Needs.** Deletion effects with counts, `bulk_delete` signal, pending handling.
**Milestone.** v0.1, using the host's native prompt for `pending`.

### UC-04: A task-scoped session

**Situation.** A developer says: "Fix the login bug. Do not change the database schema or any migrations."

**Today's gap.** No host can express constraints that apply only to one task.

**With Shiin.** The developer confirms a [task declaration](../glossary.md#task-declaration)
that limits writes to the authentication code and forbids `db.schema_change`.
Actions outside the task are denied with reason code `deny.intent`, even where standing policy would allow them.
When the task expires, its constraints and any grants tied to it end.

**Needs.** Intent binding, task lifecycle, counters.
**Milestone.** v0.2 (experimental); SQL classification in v0.5.

### UC-05: Approve consequential operations

**Situation.** Some operations are sometimes right and always consequential:
`git push --force`, `terraform apply`, `kubectl delete`, publishing a package.

**Today's gap.** Approvals are dialogs tied to one application, leave little record, and cannot be answered from elsewhere.

**With Shiin.** Policy marks these operations `pending`.
An [approval request](../spec/approval-protocol.md) shows the normalized action, not the agent's description of it.
The approver can allow it once, allow it for a scope and a time, or deny it.
Every step is recorded.

**Needs.** Approval protocol, grants, `destructive_vcs` signal, CLI approver.
**Milestone.** v0.3.

### UC-06: Review evidence afterward

**Situation.** Something went wrong during an agent session, or a lead wants to review what agents did this week.

**Today's gap.** Host transcripts show what the model said, not what was authorized and why.

**With Shiin.** The [audit log](../spec/audit-events.md) records every request, decision, approval, and grant.
`shiin verify` checks that the log's hash chain is intact.
`shiin replay` re-evaluates past requests, including against a proposed new policy, to show what would have changed.

**Needs.** Audit events, receipts, replay.
**Milestone.** v0.1 for the audit log; v0.4 for receipts, verification, and replay.

### UC-07: An unattended agent in CI

**Situation.** An agent runs in a CI job to update dependencies. No human is watching.

**Today's gap.** Hosts either prompt, which hangs the job, or run with approvals disabled.

**With Shiin.** Unattended mode turns every `pending` into `deny`, so the job fails fast and explains why.
The policy is tested in CI with `shiin policy test` against recorded cases before it is used.

**Needs.** Unattended mode, policy testing, deterministic evaluation.
**Milestone.** v0.2.

### UC-08: Govern MCP tools

**Situation.** An agent uses MCP servers for issue tracking, databases, and cloud APIs.
Some servers are third-party.

**Today's gap.** Tool annotations such as `readOnlyHint` come from the server and cannot be trusted.

**With Shiin.** The MCP proxy sits between the client and each server and maps tool calls to action requests.
Annotations can escalate a decision but never relax one.

**Needs.** MCP proxy for spec revision 2026-07-28, MCP tool mapping.
**Milestone.** v0.5.

### UC-09: Shared team policy

**Situation.** A team wants one policy, central evidence, and approvals routed to the right people.

**With Shiin.** This is not planned before 1.0.
The design keeps the constraints a team server will need,
such as signed policy bundles and transport-independent APIs;
see [Deployment models](../architecture/deployment-models.md).

**Milestone.** After 1.0.

## When Shiin is the wrong tool

| Goal | Better fit |
|---|---|
| Run untrusted code safely | An OS sandbox, container, or virtual machine |
| Stop data leaving through channels that are allowed | Egress controls or a DLP proxy |
| Detect prompt injection in content | A prompt-injection scanner |
| Prove compliance with a regulation | Shiin can supply evidence, but it makes no compliance claims |

## Read next

- [How Shiin works](../concepts/how-shiin-works.md)
- [Roadmap](../project/roadmap.md)
