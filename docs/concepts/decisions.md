<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Decisions

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

A [decision](../spec/decision.md) is the [engine](../glossary.md#engine)'s answer to an [action request](../glossary.md#action-request).

## Values

| Value | Meaning |
|---|---|
| `allow` | The action may proceed. |
| `deny` | The action must not proceed. |
| `pending` | The action must wait for an [approver](../glossary.md#approver). |

## Reason codes

Every decision carries a stable, machine-readable reason code, such as `deny.policy` or `pending.opaque`.

## How decisions are reached

The engine runs five stages in order: identity, capability, policy, intent, and risk.
A `deny` from any stage wins.
When no rule matches, the decision is `deny`.

## Grants

Only a policy rule, or a [grant](../glossary.md#grant) created by an approver, can produce `allow`.
A grant never overrides `deny`.

## Read next

- [Decision](../spec/decision.md)
- [Evaluation](../spec/evaluation.md)