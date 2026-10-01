<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.2 reviewed=2026-10-01 -->

# Cursor

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.2. Nothing on this page works yet.

## Enforcement level

L1 Cooperative.
The host honors the decision through its hook mechanism.

## Interception

Shiin uses Cursor's hooks, including `beforeShellExecution`, `beforeMCPExecution`, `beforeReadFile`, and `preToolUse`.
Hooks return allow, deny, or ask responses and support a `failClosed` option.

## What it cannot see

- Actions that do not pass through the hook.
- Code run by an allowed command.

## Read next

- [Adapter contract](../spec/adapter-contract.md)
- [Enforcement levels](../security/enforcement-levels.md)