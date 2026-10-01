<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Writing policies

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

Policy is written in TOML and compiled to Cedar.

## Structure

A policy file has a top-level `default` key and a `rules` array.

```toml
# .shiin/policy.toml (illustrative)
default = "deny"

[[rule]]
id = "edit-source"
effect = "allow"
action_types = ["fs.read", "fs.write", "fs.create"]
resources = ["./src/**", "./tests/**"]

[[rule]]
id = "no-secrets"
effect = "deny"
action_types = ["fs.read"]
resources = ["**/.env", "**/.env.*", "~/.ssh/**"]
reason = "Secret files are not readable by agents."
```

## Rules

Each rule has an `id` and an `effect` of `allow`, `deny`, or `pending`.
Rules may match on `action_types`, `resources`, `commands`, `agents`, `sessions`, and `tasks`.

## Default

When no rule matches, the decision is `deny`.
The default can be configured to `pending`, but it can never be `allow`.

## Testing

Test your policy with `shiin policy test` against recorded cases.

## Read next

- [Policy syntax](../spec/policy-syntax.md)
- [Testing policies](../guides/testing-policies.md)