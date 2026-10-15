<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-15 -->

# Claude Code

> [!NOTE]
> **Guide: draft.**
> The source build implements the current behavior below; remaining design work is marked explicitly.

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
3. Configure the host using the [quickstart](../../getting-started/quickstart-claude-code.md).
   `shiin hook claude-code` reads one JSON payload from standard input; it is not an interactive setup command.

## Read next

- [Adapter contract](../../spec/adapter-contract.md)
- [Enforcement levels](../../security/enforcement-levels.md)
