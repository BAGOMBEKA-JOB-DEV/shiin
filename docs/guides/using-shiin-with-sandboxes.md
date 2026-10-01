<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Using Shiin with sandboxes

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

Shiin decides whether an action should happen.
A sandbox limits the damage if an allowed action, or the code it runs, misbehaves.

## Layering

Shiin and a sandbox answer different questions.
Shiin decides before an action whether it should happen at all.
A sandbox limits what can happen if an allowed action, or code it runs, misbehaves.

## Why both

Shiin cannot see what a program does after it is allowed to run.
A sandbox covers that gap.
Conversely, a sandbox cannot decide that an action should not happen at all.

## Recommended setup

1. Enable Shiin's adapter for your host.
2. Enable the host's sandbox for Bash commands.
3. Keep host permission rules enabled alongside Shiin.

## Read next

- [Limitations](../concepts/limitations.md)
- [Enforcement levels](../security/enforcement-levels.md)