<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-16 -->

# Capabilities

> [!NOTE]
> **Guide: draft.**
> Explicit capability grants and refusals are implemented in source. Identity assurance and independent review remain open.

A [capability](../glossary.md#capability) is a coarse permission for an [agent](../glossary.md#agent) to attempt a whole class of [actions](../glossary.md#action) at all, for example "may run shell commands".

## How capabilities work

The capability stage is a [gate](../glossary.md#gate): it can stop or hold an action but never allow it.
Only a policy rule, or a [grant](../glossary.md#grant), can produce `allow`.

## Capabilities in policy

Capabilities are granted or denied explicitly in the policy.
There is no implicit capability.
Every action class and effect action class needs a grant; any matching refusal wins.
Capabilities pass only this gate; policy, task, risk, and built-in denials still apply.

```toml
[capabilities.shell]
id = "shell"
allow = true
action_types = ["shell.exec"]

[capabilities.read]
id = "read"
allow = true
action_types = ["fs.read"]
```

These declarations let policy consider a shell command that reads a file.
They do not grant a policy allow, and omitting the read capability denies the file-read effect.
Use `deny = true` in a separate declaration to refuse a class even when another capability grants it.
Agent selectors match asserted identifiers; they do not prove which agent is acting.

## Example

An agent with the `shell.exec` capability may attempt shell commands.
Without it, every shell command is denied with reason code `deny.capability`.

## Read next

- [Policy model](../spec/policy-model.md)
- [Agents and identity](../concepts/agents-and-identity.md)
