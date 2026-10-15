<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-15 -->

# Agents and identity

> [!NOTE]
> **Guide: draft.**
> The source build implements the current behavior below; remaining design work is marked explicitly.

An [agent](../glossary.md#agent) is software that uses a model to propose actions on a person's behalf, such as Claude Code or Codex CLI.
Shiin treats every agent as untrusted.

## Identity

An [agent identity](../glossary.md#agent-identity) is the identifier of the agent that proposed an action, together with its [assurance level](../glossary.md#assurance-level).

## Assurance levels

Current hooks report asserted identities; assurance-based identity enforcement is not implemented.
The stronger levels below are planned.

| Level | Meaning |
|---|---|
| `asserted` | The host reports the agent id. Nothing verifies it. |
| `attested-local` | The agent id is attested by a local trust root. |
| `attested-os-user` | The agent id is attested by the operating system user. |
| `attested-crypto` | The agent id is attested cryptographically. Reserved for the future. |

## Untrusted input

Anything the agent or a tool server supplies, including explanations, tool annotations, and self-reported identity, can make a decision stricter but never more permissive.

## Read next

- [Identity](../spec/identity.md)
- [Capabilities](../concepts/capabilities.md)
