<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Your first policy

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

## Create a policy

Run `shiin init` to create `.shiin/policy.toml`.

## A simple policy

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

## Validate

```console
$ shiin policy validate
```

## Test

```console
$ shiin policy test
```

## Read next

- [Writing policies](../guides/writing-policies.md)
- [Policy syntax](../spec/policy-syntax.md)