<!-- shiin-doc: kind=index status=draft implementation=none milestone=m0 reviewed=2026-09-14 -->

# Shiin documentation

> [!NOTE]
> **Project status: pre-alpha.**
> Shiin is in its specification phase.
> No code has been released, and nothing described in this documentation can be installed or run yet.

Shiin is an agent-agnostic authorization layer for AI agent actions.
It sits between AI agents (Claude Code, Codex CLI, Cursor, MCP clients, and custom agents)
and the things they act on: files, commands, networks, and services.
For every action an agent proposes, Shiin decides whether to allow it, deny it,
or hold it for a human to approve, and it records evidence of that decision.

Shiin's decisions are made by explicit, deterministic policy, never by a language model.
Policies are written once and apply to every agent.
Policies can also be bound to the specific task a developer gave the agent,
so an action can be refused because it does not belong to that task, even if it would otherwise be allowed.

## What exists, what is specified, and what is planned

| State | What it covers |
|---|---|
| **Exists today** | This documentation, the architecture decision records, the JSON Schemas and test vectors under `spec/`, and the repository's documentation tooling |
| **Specified** | The [action request](spec/action-request.md), [decision](spec/decision.md), [evaluation](spec/evaluation.md), [policy model](spec/policy-model.md), [intent binding](spec/intent-binding.md), [approval protocol](spec/approval-protocol.md), [audit events](spec/audit-events.md), [receipts](spec/receipts.md), [adapter contract](spec/adapter-contract.md), and [local API](spec/local-api.md) |
| **Planned** | Every command, adapter, daemon, and library; see the [roadmap](project/roadmap.md) for what each milestone delivers |

## Start here

| If you are | Read, in order |
|---|---|
| New to Shiin | [Vision](overview/vision.md), [How Shiin works](concepts/how-shiin-works.md), [Use cases](overview/use-cases.md), [Limitations](concepts/limitations.md) |
| Evaluating Shiin's security | [Threat model](security/threat-model.md), [Security model](security/security-model.md), [Enforcement levels](security/enforcement-levels.md), [Limitations](concepts/limitations.md) |
| Comparing Shiin with other tools | [Landscape](overview/landscape.md), [FAQ](overview/faq.md) |
| Building an adapter or integration | [Adapter contract](spec/adapter-contract.md), [Action request](spec/action-request.md), [Decision](spec/decision.md), [Local API](spec/local-api.md) |
| Planning to write policy | [Policies](concepts/policies.md), [Policy model](spec/policy-model.md), [Writing policies](guides/writing-policies.md) |
| Contributing code | [Workspace and crates](engineering/workspace-and-crates.md), [Coding standards](engineering/coding-standards.md), [Testing strategy](engineering/testing-strategy.md), [CONTRIBUTING.md](https://github.com/BAGOMBEKA-JOB-DEV/shiin/blob/main/CONTRIBUTING.md) |
| Proposing a change | [ADR index](adr/README.md), [RFC process](rfcs/README.md) |

## Reading the status banners

Every page starts with a banner that says what kind of page it is and how settled its content is.

- **Specification** pages are normative contracts. Implementations will be tested against them.
- **Design document** pages explain the intended design and the reasons behind it.
- **Design intent: not implemented** pages describe what using Shiin will look like.
  They are written ahead of the code so the experience can be reviewed before it is built.
  Nothing on those pages works yet.
- **Architecture decision** pages record one decision and its status.
- **Project policy** pages set rules for the project and its contributors.

The [documentation style guide](project/docs-style-guide.md) defines every label and the checks that keep banners accurate.

## Giving feedback

The specification phase is the cheapest time to change the design.

- For a problem in a specification, open an issue using the spec feedback form.
- For anything that might be a way to bypass Shiin, do not open a public issue.
  Follow [SECURITY.md](https://github.com/BAGOMBEKA-JOB-DEV/shiin/blob/main/SECURITY.md) instead.
- For larger changes, read the [RFC process](rfcs/README.md).
