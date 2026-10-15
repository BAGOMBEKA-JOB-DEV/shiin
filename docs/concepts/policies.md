<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-15 -->

# Policies

> [!NOTE]
> **Guide: draft.**
> The source build implements the current behavior below; remaining design work is marked explicitly.

A [policy](../glossary.md#policy) is the complete set of [rules](../glossary.md#rule), [capabilities](../glossary.md#capability), defaults, and risk configuration that applies to an evaluation.

## Layers

The designed layers are built-in, user, and workspace.
The current CLI loads `.shiin/policy.toml` as one operator-controlled layer; separate user/workspace loading and trust-by-hash are not implemented.
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
