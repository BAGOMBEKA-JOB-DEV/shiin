<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Aider in a sandbox

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

## Enforcement level

L1 Cooperative through a shell wrapper.

## Setup

Run Aider inside a sandbox such as a container or virtual machine, with Shiin's shell wrapper intercepting commands.

## What it cannot see

- Commands typed directly at the shell without the wrapper.

## Read next

- [Using Shiin with sandboxes](../using-shiin-with-sandboxes.md)
- [Shell wrapper](../architecture/overview.md)