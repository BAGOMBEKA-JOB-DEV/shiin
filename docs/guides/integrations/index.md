<!-- shiin-doc: kind=index status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# Integrations

> [!NOTE]
> **Project status: pre-alpha.**
> These integration guides are design intent. No adapter works yet.

Adapters connect a host — Claude Code, Codex CLI, Cursor, an MCP client, or a custom agent — to the Shiin engine.
Each integration page documents the host's interception mechanism, the enforcement level it can provide, and what it cannot see.

The [adapter contract](../../spec/adapter-contract.md) is the same for every host.
Integration pages only describe the host-specific wiring, not the contract.

## Available integrations

| Host | Enforcement level | Status |
|---|---|---|
| [Claude Code](claude-code.md) | L1 cooperative | Planned v0.1 |
| [Codex CLI](codex.md) | L1 cooperative | Planned v0.2 |
| [Cursor](cursor.md) | L1 cooperative | Planned v0.2 |
| [MCP proxy](mcp-proxy.md) | L2 mediated | Planned v0.5 |
| [Custom agents](custom-agents.md) | L1/L2 via API | Planned v0.5 |
| [Aider in a sandbox](aider-sandboxed.md) | L1 cooperative | Planned v0.6 |

## Writing a new integration

Third parties can integrate any host that exposes a pre-execution hook or that can route tool calls through the [MCP proxy](mcp-proxy.md).
See the [adapter contract](../../spec/adapter-contract.md) and [local API](../../spec/local-api.md) specifications.

## Read next

- [Adapter contract](../../spec/adapter-contract.md)
- [Enforcement levels](../../security/enforcement-levels.md)
- [Landscape](../../overview/landscape.md)
