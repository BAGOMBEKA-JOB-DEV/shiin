<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.2 reviewed=2026-10-01 -->

# Codex CLI

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.2. Nothing on this page works yet.

## Enforcement level

L1 Cooperative.
The host honors the decision through its hook mechanism.
Codex documentation calls hooks "a useful guardrail, not a complete enforcement boundary."

## Interception

Shiin uses Codex CLI's `PreToolUse` hooks covering Bash, `apply_patch`, and MCP tools.

## What it cannot see

- Actions that do not pass through the hook.
- Code run by an allowed command.

## Read next

- [Adapter contract](../spec/adapter-contract.md)
- [Enforcement levels](../security/enforcement-levels.md)