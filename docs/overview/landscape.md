<!-- shiin-doc: kind=explanation status=draft implementation=none milestone=n/a reviewed=2026-09-14 -->

# Landscape

> [!NOTE]
> **Design document: draft.**
> Describes where Shiin's intended design fits among existing tools. No implementation of Shiin exists yet.
> Every statement about another project links its source and gives the date it was verified.

## Summary

Many tools already help people control what AI [agents](../glossary.md#agent) do.
They intercept [actions](../glossary.md#action) at different points, enforce decisions with different strength, and leave different evidence.
Shiin is designed as a local, agent-agnostic authorization layer for agent actions.
It is not a sandbox, a network firewall, or a gateway, and it is designed to work alongside all three.

Deciding whether an agent's action proceeds, is refused, or waits for a person is not a new idea.
Several open-source projects already do it, and [Open-source agent action gates](#open-source-agent-action-gates) credits them with sources.
Shiin's case rests on the four differentiators described in the [vision](vision.md#what-makes-shiin-different):
task-intent binding, an open agent-neutral contract, analyzable policy, and explicit enforcement levels.
Shiin exists only as a specification.
If you need a working tool today, evaluate the projects on this page.

### How to read this page

- Facts about other projects come from their own documentation or repositories, linked beside each statement, as checked on 2026-09-14.
- A table cell that reads "not documented" means the sources checked for this page did not establish the fact.
  It does not mean the project lacks the capability.
- Each project is described in its own terms.
  Where a project's decision labels differ from Shiin's `allow`, `deny`, and `pending`, they are quoted as that project's labels.
- Statements about Shiin describe its design, not shipped features.

## Standards that frame the problem

Shiin's [problem statement](vision.md#the-problem) draws on three public documents, which are also useful when comparing any tool on this page:

- [OWASP LLM06:2025 Excessive Agency](https://genai.owasp.org/llmrisk/llm062025-excessive-agency/) (verified 2026-09-14).
- [OWASP Top 10 for Agentic Applications 2026](https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/) (verified 2026-09-14).
- The NIST National Cybersecurity Center of Excellence [concept paper on software and AI agent identity and authorization](https://csrc.nist.gov/pubs/other/2026/02/05/accelerating-the-adoption-of-software-and-ai-agent/ipd), published in February 2026 (verified 2026-09-14).

## Categories

Tools in this space fall into six broad categories, and a single tool can span more than one.
The categories act at different points and answer different questions, which is why they combine well.

| Category | Where it acts | Question it answers |
|---|---|---|
| [Host-native controls](#host-native-controls) | Inside the host, before a tool runs | What does this host let the agent do? |
| [OS sandboxes and isolation](#os-sandboxes-and-isolation) | Around a running process | What can this process reach, whatever it does? |
| [Network egress controls](#network-egress-controls) | On outbound traffic | Where may traffic go? |
| [MCP and agent gateways](#mcp-and-agent-gateways) | Between an agent and the tool servers or other agents it calls | Which tools and agents may be called? |
| [Policy engines as building blocks](#policy-engines-as-building-blocks) | Inside other tools | How are rules written and evaluated? |
| [Open-source agent action gates](#open-source-agent-action-gates) | Where an agent proposes an action | Should this action proceed, be refused, or wait for a person? |

### Host-native controls

The applications that run agents ship their own controls, and they are the first thing to configure.

- **Claude Code** has permission rules, and `PreToolUse` and `PermissionRequest` [hooks](../glossary.md#hook) that can return allow, deny, or ask decisions
  ([hooks](https://code.claude.com/docs/en/hooks), [permissions](https://code.claude.com/docs/en/permissions), verified 2026-09-14).
  Hooks can be disabled with `--bare` or the `disableAllHooks` setting, and managed hooks cannot be disabled ([hooks](https://code.claude.com/docs/en/hooks), verified 2026-09-14).
- **OpenAI Codex CLI** has approval policies and sandbox modes ([agent approvals and security](https://learn.chatgpt.com/docs/agent-approvals-security), verified 2026-09-14).
  Its `PreToolUse` hooks cover Bash, `apply_patch`, and MCP tools ([hooks](https://learn.chatgpt.com/docs/hooks), verified 2026-09-14).
  Its documentation calls hooks "a useful guardrail, not a complete enforcement boundary" ([hooks](https://learn.chatgpt.com/docs/hooks), verified 2026-09-14).
- **Cursor** has offered hooks since version 1.7, including `beforeShellExecution`, `beforeMCPExecution`, `beforeReadFile`, and `preToolUse`,
  with allow, deny, or ask responses and a `failClosed` option ([hooks](https://cursor.com/docs/hooks), verified 2026-09-14).

**How Shiin relates.**
Shiin's [enforcement level](../glossary.md#enforcement-level) L1 adapters are designed to use these same hook mechanisms, so they inherit the hooks' strengths and limits.
Shiin is designed to add to host controls, not to replace them, so keep them enabled.
[Enforcement levels](../security/enforcement-levels.md) records what each adapter can and cannot see.

### OS sandboxes and isolation

A sandbox limits what a process can do after it starts, whatever the reason it was started.
Containers and virtual machines are the general-purpose form, and some hosts include a sandbox of their own:

- Claude Code's sandbox applies to Bash commands and their child processes ([sandboxing](https://code.claude.com/docs/en/sandboxing), verified 2026-09-14).
- Codex CLI's sandbox modes use Seatbelt on macOS, and bwrap and seccomp on Linux ([agent approvals and security](https://learn.chatgpt.com/docs/agent-approvals-security), verified 2026-09-14).

**How Shiin relates.**
Shiin decides whether an action should happen; a sandbox limits the damage if an allowed action, or the code it runs, misbehaves.
Shiin cannot see what a program does after it is allowed to run, as [Limitations](../concepts/limitations.md) explains, so a sandbox covers a gap that Shiin leaves.
OS-enforced integration (L3) is a future enforcement level.
[Using Shiin with sandboxes](../guides/using-shiin-with-sandboxes.md) describes the intended layering.

### Network egress controls

Egress controls restrict where traffic can go, regardless of which action produced it.

- The **GitHub Copilot coding agent firewall** restricts outbound network access for processes started by the agent's Bash tool
  ([customize the agent firewall](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-firewall), verified 2026-09-14).
  It does not apply to MCP servers or setup steps, and its documentation says it "should not be considered a comprehensive security solution"
  ([customize the agent firewall](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-firewall), verified 2026-09-14).
- **Coder Agent Firewall**, formerly called Agent Boundaries, is a process-level network firewall that restricts domains and HTTP methods and keeps audit logs
  ([agent firewall](https://coder.com/docs/ai-coder/agent-firewall), verified 2026-09-14).
  It covers network egress only ([agent firewall](https://coder.com/docs/ai-coder/agent-firewall), verified 2026-09-14).
- **pipelock** is an agent firewall for HTTP, MCP, A2A, and WebSocket traffic,
  with DLP, SSRF, prompt-injection, and tool-poisoning detection and signed action receipts
  ([luckyPipewrench/pipelock](https://github.com/luckyPipewrench/pipelock), verified 2026-09-14).
  It is written in Go, with an Apache-2.0 core and Elastic-2.0 enterprise features ([luckyPipewrench/pipelock](https://github.com/luckyPipewrench/pipelock), verified 2026-09-14).

**How Shiin relates.**
Shiin's design can deny a network action it can see, but it does not inspect traffic content,
and it cannot see data leaving through an action it has allowed.
Egress controls are the right layer for that, and Shiin is designed to run alongside them.

### MCP and agent gateways

Gateways sit in the traffic between an agent and the tool servers or other agents it calls.

- **agentgateway** is a Linux Foundation proxy for MCP and A2A with RBAC rules written in CEL
  ([agentgateway/agentgateway](https://github.com/agentgateway/agentgateway), verified 2026-09-14).
  It is written in Rust and licensed Apache-2.0 ([agentgateway/agentgateway](https://github.com/agentgateway/agentgateway), verified 2026-09-14).
- **Invariant** provides rule-based guardrails for LLM and MCP applications
  ([invariantlabs-ai/invariant](https://github.com/invariantlabs-ai/invariant), verified 2026-09-14).
  It is written in Python and licensed Apache-2.0 ([invariantlabs-ai/invariant](https://github.com/invariantlabs-ai/invariant), verified 2026-09-14).
  Invariant Labs was acquired by Snyk in June 2025
  ([Snyk announcement](https://snyk.io/news/snyk-acquires-invariant-labs-to-accelerate-agentic-ai-security-innovation/), verified 2026-09-14).
- **Belay**, described under [open-source agent action gates](#open-source-agent-action-gates), can also work as an MCP proxy.

**How Shiin relates.**
Shiin's MCP proxy (planned: v0.5) is an L2 mediated adapter with a narrower role than a general gateway.
It is designed to turn MCP tool calls into [action requests](../glossary.md#action-request) and apply the same policy that hook adapters use.
MCP tool annotations can make a decision stricter but never more permissive.
If what you need is access control for MCP or A2A traffic shared across many agents, a general gateway may be the better fit.

### Policy engines as building blocks

Policy engines evaluate rules.
They do not intercept agent actions themselves; other tools embed them.

- **Cedar** is an authorization policy language and engine ([cedar-policy/cedar](https://github.com/cedar-policy/cedar), verified 2026-09-14).
  Its `cedar-policy` Rust crate is licensed Apache-2.0, Cedar is a CNCF Sandbox project, and it is formally verified in Lean
  ([cedar-policy/cedar](https://github.com/cedar-policy/cedar), verified 2026-09-14).
- **regorus** is a Rust interpreter for Rego, and it is used in Azure Policy ([microsoft/regorus](https://github.com/microsoft/regorus), verified 2026-09-14).
- Other projects on this page also build on policy languages:
  Cupcake uses OPA/Rego compiled to WebAssembly ([eqtylab/cupcake](https://github.com/eqtylab/cupcake), verified 2026-09-14),
  and agentgateway uses CEL ([agentgateway/agentgateway](https://github.com/agentgateway/agentgateway), verified 2026-09-14).

**How Shiin relates.**
Shiin builds on Cedar rather than competing with it.
Operators write TOML policy, which Shiin compiles deterministically to Cedar, and advanced users can write Cedar directly.
The choice is proposed in [ADR-0014](../adr/0014-cedar-policy-engine-with-toml-front-end.md) and confirmed only by a spike; the fallback is a native Rust evaluator behind the same policy model.

### Open-source agent action gates

These projects are closest to Shiin.
They act at the point where an agent proposes an action and decide what happens to it.

- **Belay** is a local-first security layer at the tool-call boundary for Claude Code, Codex, Cursor, and MCP, using native hooks or an MCP proxy
  ([SECBLOK/belay](https://github.com/SECBLOK/belay), verified 2026-09-14).
  It returns "deny", "ask", or "allow" and keeps a tamper-evident audit trail ([SECBLOK/belay](https://github.com/SECBLOK/belay), verified 2026-09-14).
  It is written in Rust and licensed AGPL-3.0, with a commercial option ([SECBLOK/belay](https://github.com/SECBLOK/belay), verified 2026-09-14).
- **Cupcake** works through the hooks of Claude Code, Cursor, and other agents, with policies written in OPA/Rego and compiled to WebAssembly
  ([eqtylab/cupcake](https://github.com/eqtylab/cupcake), verified 2026-09-14).
  It is written in Rust and licensed Apache-2.0 ([eqtylab/cupcake](https://github.com/eqtylab/cupcake), verified 2026-09-14).
- **AgentFence** is deny-by-default, with "allow", "deny", and "ask" outcomes and Ed25519-signed, hash-chained logs
  ([dgenio/agentfence](https://github.com/dgenio/agentfence), verified 2026-09-14).
  It is written in Go and licensed Apache-2.0 ([dgenio/agentfence](https://github.com/dgenio/agentfence), verified 2026-09-14).
- **Cerberus** is a local gateway for Claude Code, Codex, Cursor, and Cline that risk-scores actions
  and returns one of the project's own labels: "ALLOW", "AUDIT", "HITL", or "BLOCK"
  ([Adirdabush1/cerberus](https://github.com/Adirdabush1/cerberus), verified 2026-09-14).
  It is written in TypeScript and licensed Apache-2.0 ([Adirdabush1/cerberus](https://github.com/Adirdabush1/cerberus), verified 2026-09-14).
- **agent-belay** is a desktop approval app that shows tool requests with diffs and command previews for Claude Code, Copilot, and others,
  with optional LLM-based risk scoring ([mikepenz/agent-belay](https://github.com/mikepenz/agent-belay), verified 2026-09-14).
  It is written in Kotlin and licensed Apache-2.0 ([mikepenz/agent-belay](https://github.com/mikepenz/agent-belay), verified 2026-09-14).
  It is a separate repository from SECBLOK/belay.
- **ActionRail** checks consequential tool-call arguments against live systems of record
  and returns one of the project's own labels: "allow", "hold", or "block"
  ([ToolJet/ActionRail](https://github.com/ToolJet/ActionRail), verified 2026-09-14).
  It is written in Python, licensed Apache-2.0, and in public beta ([ToolJet/ActionRail](https://github.com/ToolJet/ActionRail), verified 2026-09-14).
- **latchagent/latch** describes itself as a "control layer for autonomous AI agents" in which risky actions wait for approval
  ([latchagent/latch](https://github.com/latchagent/latch), verified 2026-09-14).
  It is written in TypeScript and licensed MIT ([latchagent/latch](https://github.com/latchagent/latch), verified 2026-09-14).
- **Agent Governance Toolkit** provides policy enforcement, zero-trust identity, sandboxing, and audit for AI agents
  ([microsoft/agent-governance-toolkit](https://github.com/microsoft/agent-governance-toolkit), verified 2026-09-14).
  It is written in Python and licensed MIT ([microsoft/agent-governance-toolkit](https://github.com/microsoft/agent-governance-toolkit), verified 2026-09-14).

**How Shiin relates.**
This is where Shiin overlaps most, as the section [Where Shiin overlaps and where it differs](#where-shiin-overlaps-and-where-it-differs) explains.

## Comparison dimensions

These questions help compare any tool on this page, including Shiin.
The last column states Shiin's design, which is not implemented.

| Dimension | Question to ask of any tool | Shiin's design |
|---|---|---|
| Agent-agnostic | Does one configuration apply across several agents and hosts? | Policy refers to normalized [action types](../spec/action-types.md), not host tool names, and each host gets an [adapter](../glossary.md#adapter) ([ADR-0025](../adr/0025-adapter-roadmap.md)) |
| Interception point and enforcement level | Where is the action caught, and can the agent route around that point? | Every adapter is labeled L1 cooperative, L2 mediated, or L3 OS-enforced, together with what it cannot see ([Enforcement levels](../security/enforcement-levels.md)) |
| Decision determinism | Do the same inputs always produce the same decision, and can model output change it? | The same action request and policy snapshot always produce the same decision; no language-model or machine-learning output is an input ([ADR-0009](../adr/0009-deterministic-enforcement.md)) |
| Approval model | How does a person approve, and is the approval format specified beyond one application? | A versioned [approval protocol](../spec/approval-protocol.md) with single-use and scoped grants (planned: v0.3) |
| Evidence | What record remains, and can someone else verify it? | A hash-chained [audit log](../glossary.md#audit-log) (planned: v0.1), then signed checkpoints and [receipts](../glossary.md#receipt) (planned: v0.4) |
| Policy portability | Can policy be reviewed, tested, and reused outside one host? | TOML compiled to Cedar, with raw Cedar allowed ([ADR-0014](../adr/0014-cedar-policy-engine-with-toml-front-end.md), proposed) |
| Task-intent binding | Can a decision depend on the constraints of the current task, not only on standing rules? | Structured, human-confirmed [task declarations](../glossary.md#task-declaration) (planned: v0.2, experimental) |
| Local-first | Does it run on the developer's machine without a hosted service or telemetry? | Runs locally with no telemetry, and every network feature is opt-in ([ADR-0007](../adr/0007-no-telemetry.md)) |

## Projects at a glance

The tables below summarize the facts cited in [Categories](#categories).
Every row links its source, and every fact was verified on 2026-09-14.
"Not documented" means the sources checked for this page did not establish the fact.

### Host and platform controls

| Control | What it does (per its documentation) | Limits stated in its documentation |
|---|---|---|
| Claude Code ([hooks](https://code.claude.com/docs/en/hooks), [permissions](https://code.claude.com/docs/en/permissions), [sandboxing](https://code.claude.com/docs/en/sandboxing); verified 2026-09-14) | Permission rules; `PreToolUse` and `PermissionRequest` hooks with allow, deny, or ask decisions; a sandbox for Bash commands and their child processes | Hooks can be disabled with `--bare` or `disableAllHooks`; managed hooks cannot be disabled |
| OpenAI Codex CLI ([agent approvals and security](https://learn.chatgpt.com/docs/agent-approvals-security), [hooks](https://learn.chatgpt.com/docs/hooks); verified 2026-09-14) | Approval policies; sandbox modes using Seatbelt on macOS, and bwrap and seccomp on Linux; `PreToolUse` hooks covering Bash, `apply_patch`, and MCP tools | Hooks are "a useful guardrail, not a complete enforcement boundary" |
| Cursor ([hooks](https://cursor.com/docs/hooks); verified 2026-09-14) | Hooks since 1.7, including `beforeShellExecution`, `beforeMCPExecution`, `beforeReadFile`, and `preToolUse`, with allow, deny, or ask responses and a `failClosed` option | not documented |
| GitHub Copilot coding agent firewall ([customize the agent firewall](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-firewall); verified 2026-09-14) | Restricts outbound network access for processes started by the agent's Bash tool | Does not apply to MCP servers or setup steps; "should not be considered a comprehensive security solution" |
| Coder Agent Firewall ([agent firewall](https://coder.com/docs/ai-coder/agent-firewall); verified 2026-09-14) | Process-level network firewall that restricts domains and HTTP methods, with audit logs; formerly called Agent Boundaries | Network egress only |

### Open-source projects

| Project | What it does (per its documentation) | Interception point | Decision outcomes (project's own labels) | Evidence | Policy language | Language and license |
|---|---|---|---|---|---|---|
| [SECBLOK/belay](https://github.com/SECBLOK/belay) (verified 2026-09-14) | Local-first security layer at the tool-call boundary for Claude Code, Codex, Cursor, and MCP | Native hooks or an MCP proxy | "deny", "ask", "allow" | Tamper-evident audit trail | not documented | Rust; AGPL-3.0 with a commercial option |
| [eqtylab/cupcake](https://github.com/eqtylab/cupcake) (verified 2026-09-14) | Policies for Claude Code, Cursor, and other agents | Host hooks | not documented | not documented | OPA/Rego compiled to WebAssembly | Rust; Apache-2.0 |
| [dgenio/agentfence](https://github.com/dgenio/agentfence) (verified 2026-09-14) | Deny-by-default action decisions | not documented | "allow", "deny", "ask" | Ed25519-signed, hash-chained logs | not documented | Go; Apache-2.0 |
| [Adirdabush1/cerberus](https://github.com/Adirdabush1/cerberus) (verified 2026-09-14) | Risk-scores actions from Claude Code, Codex, Cursor, and Cline | Local gateway | "ALLOW", "AUDIT", "HITL", "BLOCK" | not documented | not documented | TypeScript; Apache-2.0 |
| [mikepenz/agent-belay](https://github.com/mikepenz/agent-belay) (verified 2026-09-14) | Desktop approval app showing tool requests with diffs and command previews for Claude Code, Copilot, and others; optional LLM-based risk scoring | not documented | not documented | not documented | not documented | Kotlin; Apache-2.0 |
| [ToolJet/ActionRail](https://github.com/ToolJet/ActionRail) (verified 2026-09-14) | Checks consequential tool-call arguments against live systems of record; public beta | not documented | "allow", "hold", "block" | not documented | not documented | Python; Apache-2.0 |
| [latchagent/latch](https://github.com/latchagent/latch) (verified 2026-09-14) | "Control layer for autonomous AI agents" in which risky actions wait for approval | not documented | not documented | not documented | not documented | TypeScript; MIT |
| [microsoft/agent-governance-toolkit](https://github.com/microsoft/agent-governance-toolkit) (verified 2026-09-14) | Policy enforcement, zero-trust identity, sandboxing, and audit for AI agents | not documented | not documented | Audit | not documented | Python; MIT |
| [luckyPipewrench/pipelock](https://github.com/luckyPipewrench/pipelock) (verified 2026-09-14) | Agent firewall with DLP, SSRF, prompt-injection, and tool-poisoning detection | HTTP, MCP, A2A, and WebSocket traffic | not documented | Signed action receipts | not documented | Go; Apache-2.0 core, Elastic-2.0 enterprise features |
| [agentgateway/agentgateway](https://github.com/agentgateway/agentgateway) (verified 2026-09-14) | Linux Foundation proxy for MCP and A2A | MCP and A2A proxy | not documented | not documented | RBAC rules in CEL | Rust; Apache-2.0 |
| [invariantlabs-ai/invariant](https://github.com/invariantlabs-ai/invariant) (verified 2026-09-14) | Rule-based guardrails for LLM and MCP applications | not documented | not documented | not documented | not documented | Python; Apache-2.0 |

## Where Shiin overlaps and where it differs

### Overlap

Deciding, before an agent's action runs, whether it proceeds, is refused, or waits for a person already exists in open-source projects.

- Belay and AgentFence document allow, deny, and ask outcomes together with tamper-evident or signed, hash-chained audit records
  ([SECBLOK/belay](https://github.com/SECBLOK/belay), [dgenio/agentfence](https://github.com/dgenio/agentfence), verified 2026-09-14).
- Cupcake applies policy through the hooks of Claude Code, Cursor, and other agents ([eqtylab/cupcake](https://github.com/eqtylab/cupcake), verified 2026-09-14).
- Cerberus returns its own "ALLOW", "AUDIT", "HITL", or "BLOCK" labels from a local gateway ([Adirdabush1/cerberus](https://github.com/Adirdabush1/cerberus), verified 2026-09-14).

Related ideas exist too.
Cupcake and agentgateway express policy in established languages, pipelock issues signed action receipts, and Belay is local-first and written in Rust (sources in [Categories](#categories)).
None of these, on its own, is a reason to choose Shiin:
intercepting actions through hooks, returning allow-or-deny outcomes, keeping an audit trail, writing policy as code, running locally, or being written in Rust.

### Differences

Shiin's case rests on the four differentiators in the [vision](vision.md#what-makes-shiin-different).
Each one is a design commitment, and none is implemented.

1. **Task-intent binding.**
   Shiin is designed to check each action against the constraints of a human-confirmed [task declaration](../glossary.md#task-declaration), not only against standing policy (planned: v0.2, experimental).
   This page records no verified fact that another project listed here binds decisions to per-task constraints.
   If one does, please request a correction.
2. **An open, agent-neutral contract.**
   The [action request](../spec/action-request.md), [decision](../spec/decision.md), and [approval protocol](../spec/approval-protocol.md) are designed as versioned specifications with schemas and test vectors that other tools can implement.
   Several projects above support more than one host, but supporting several hosts is not the same as publishing the request and approval formats as independent specifications.
   This page records no verified fact that another project listed here publishes such specifications.
3. **Analyzable policy.**
   Policy as code is not new; Cupcake uses Rego and agentgateway uses CEL.
   Shiin's difference is its proposed choice of Cedar, a formally verified authorization language, behind a TOML front-end, so that policies can be tested, explained, and reasoned about ([ADR-0014](../adr/0014-cedar-policy-engine-with-toml-front-end.md), proposed).
4. **Explicit enforcement levels.**
   Vendors already state the limits of their own controls:
   Codex CLI calls hooks "a useful guardrail, not a complete enforcement boundary" ([hooks](https://learn.chatgpt.com/docs/hooks), verified 2026-09-14),
   and GitHub says its agent firewall "should not be considered a comprehensive security solution"
   ([customize the agent firewall](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-firewall), verified 2026-09-14).
   Shiin applies that honesty uniformly: every adapter is designed to state its [enforcement level](../security/enforcement-levels.md) and what it cannot see ([ADR-0033](../adr/0033-enforcement-levels.md)).

A differentiator on paper is not a working tool.
Until Shiin ships, the projects above are the ones you can use.

## When to use something else instead of or alongside Shiin

| If you need | Consider | Relationship to Shiin |
|---|---|---|
| A tool you can use today | One of the [open-source agent action gates](#open-source-agent-action-gates) | Instead, for now: Shiin has no implementation |
| Controls for one host, where its built-in settings meet your needs | [Host-native controls](#host-native-controls) | Instead or alongside: keep them enabled even if you adopt Shiin |
| To limit what an allowed command, or the code it runs, can reach | [An OS sandbox, container, or virtual machine](#os-sandboxes-and-isolation) | Alongside: Shiin does not isolate processes |
| To restrict where an agent's traffic can go, or to inspect that traffic | [Network egress controls](#network-egress-controls) | Alongside: Shiin does not inspect traffic and cannot see data leaving through an allowed action |
| Access control for MCP or A2A traffic shared across many agents | [MCP and agent gateways](#mcp-and-agent-gateways) | Instead or alongside, depending on whether you also want per-action policy and task binding on the developer's machine |
| A desktop approval app with diffs and command previews | [mikepenz/agent-belay](https://github.com/mikepenz/agent-belay) (verified 2026-09-14) | Instead or alongside: Shiin's approval protocol is planned for v0.3, and its first approver is a command-line tool |
| To check tool-call arguments against live systems of record | [ToolJet/ActionRail](https://github.com/ToolJet/ActionRail) (verified 2026-09-14) | Alongside: Shiin's engine performs no I/O and decides from the action request and local policy alone |
| Policies written in Rego | [eqtylab/cupcake](https://github.com/eqtylab/cupcake), or [microsoft/regorus](https://github.com/microsoft/regorus) to build your own (verified 2026-09-14) | Instead: Shiin proposes Cedar |
| A broad agent-governance toolkit in Python | [microsoft/agent-governance-toolkit](https://github.com/microsoft/agent-governance-toolkit) (verified 2026-09-14) | Instead or alongside |
| An authorization engine inside your own application, without agent-specific features | [Cedar](https://github.com/cedar-policy/cedar) directly (verified 2026-09-14) | Instead: Shiin adds action types, adapters, task binding, approvals, and evidence on top of Cedar |

## Keeping this page accurate

- Anyone, including the maintainers of a project described here, may request a correction by opening an issue at
  [BAGOMBEKA-JOB-DEV/shiin](https://github.com/BAGOMBEKA-JOB-DEV/shiin/issues).
- Every fact about another project links a public source and states the date it was verified.
- A claim that cannot be verified from a public source is removed rather than guessed, and a table cell that cannot be verified reads "not documented".
- Maintainers re-verify every fact on this page before each release and update the `reviewed` date in the page metadata.
- Projects are described in their own terms and credited for their work. This page does not disparage any project.
- A project is added through a pull request that includes public sources. Being listed is not an endorsement.

## Read next

- [Vision](vision.md)
- [Limitations](../concepts/limitations.md)
- [Enforcement levels](../security/enforcement-levels.md)
- [FAQ](faq.md)
