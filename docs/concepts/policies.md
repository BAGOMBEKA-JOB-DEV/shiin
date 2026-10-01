<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Policies

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

A [policy](../glossary.md#policy) is the complete set of [rules](../glossary.md#rule), [capabilities](../glossary.md#capability), defaults, and risk configuration that applies to an evaluation.

## Layers

Policy is drawn from layers, in order: built-in, user, and workspace.
A workspace layer can restrict a user layer only; it can never make a deny into an allow unless the operator trusts it by hash.

## Rules

A rule matches an action request and produces `allow`, `deny`, or `pending`.
Policies refer to [action types](../spec/action-types.md), never to host tool names.

## Default decision

When no rule matches, the decision is `deny`.
The default can be configured to `pending`, but it can never be `allow`.

## Writing policy

Policy is written in TOML and compiled to Cedar.
See [Writing policies](../guides/writing-policies.md).

## Read next

- [Policy model](../spec/policy-model.md)
- [Policy syntax](../spec/policy-syntax.md)