<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Adapters

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

An [adapter](../glossary.md#adapter) connects a [host](../glossary.md#host) to Shiin.
It intercepts proposed actions, builds [action requests](../glossary.md#action-request), and enforces the resulting [decision](../glossary.md#decision) using the host's own protocol.
Adapters never make decisions.

## Enforcement levels

Every adapter declares an [enforcement level](../glossary.md#enforcement-level):

| Level | Meaning |
|---|---|
| L1 Cooperative | The host honors the decision through its hook mechanism. |
| L2 Mediated | Shiin sits in the data path. |
| L3 OS-enforced | The operating system or a sandbox enforces the decision. |

## Hosts

| Host | Level | Interception | Milestone |
|---|---|---|---|
| Claude Code | L1 | hooks | v0.1 |
| Codex CLI | L1 | hooks | v0.2 |
| Cursor | L1 | hooks | v0.2 |
| MCP proxy | L2 | data path | v0.5 |

## Honest labeling

Every adapter states its enforcement level and what it cannot see.

## Read next

- [Adapter contract](../spec/adapter-contract.md)
- [Integrations](../guides/integrations/claude-code.md)