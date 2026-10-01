<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Capabilities

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

A [capability](../glossary.md#capability) is a coarse permission for an [agent](../glossary.md#agent) to attempt a whole class of [actions](../glossary.md#action) at all, for example "may run shell commands".

## How capabilities work

The capability stage is a [gate](../glossary.md#gate): it can stop or hold an action but never allow it.
Only a policy rule, or a [grant](../glossary.md#grant), can produce `allow`.

## Capabilities in policy

Capabilities are granted or denied explicitly in the policy.
There is no implicit capability.

## Example

An agent with the `shell.exec` capability may attempt shell commands.
Without it, every shell command is denied with reason code `deny.capability`.

## Read next

- [Policy model](../spec/policy-model.md)
- [Agents and identity](../concepts/agents-and-identity.md)