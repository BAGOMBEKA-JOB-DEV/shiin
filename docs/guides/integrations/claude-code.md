<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Claude Code

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

## Enforcement level

L1 Cooperative.
The host honors the decision through its hook mechanism, which can be disabled with `--bare` or `disableAllHooks`.

## Interception

Shiin uses Claude Code's `PreToolUse` hook.
The hook receives the tool payload and returns allow, deny, or ask.

## What it cannot see

- Actions that do not pass through the hook.
- Code run by an allowed command.

## Setup

1. Install Shiin.
2. Run `shiin init`.
3. Run `shiin hook claude-code` and follow the instructions to configure the hook.

## Read next

- [Adapter contract](../spec/adapter-contract.md)
- [Enforcement levels](../security/enforcement-levels.md)