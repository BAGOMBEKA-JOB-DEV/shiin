<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-16 -->

# Your first policy

> [!NOTE]
> **Guide: draft.**
> The policy format and the commands below work with a source build.

`shiin init` writes a starter policy. This page explains what it says and how to change it
with confidence.

## The starter policy

```toml
version = 1

[capabilities.workspace]
id = "workspace"
allow = true
action_types = ["fs.*", "shell.exec"]

# Anything no rule allows is denied.
[defaults]
decision = "deny"

[[rule]]
id = "read-and-edit-source"
effect = "allow"
action_types = ["fs.read", "fs.list", "fs.write", "fs.create"]
resources = ["./src/**", "./tests/**", "./docs/**"]

[[rule]]
id = "no-secrets"
effect = "deny"
action_types = ["fs.read"]
resources = ["**/.env", "**/.env.*", "~/.ssh/**", "**/credentials"]
reason = "Secret files are not readable by agents."

[[rule]]
id = "run-tests"
effect = "allow"
action_types = ["shell.exec"]
commands = ["cargo test*", "cargo nextest*", "git status", "git diff*"]

[risk]
opaque_command = "pending"
secret_path = "deny"
bulk_delete = { decision = "pending", threshold = 20 }
destructive_vcs = "pending"
privilege_escalation = "pending"
```

Three things are worth noticing.

- **The default is a denial.** Every rule widens what is possible from nothing. You cannot
  forget to deny something, only forget to allow it, and the failure is a refusal rather
  than an exposure.
- **`no-secrets` wins over `read-and-edit-source`.** A denial beats an allow wherever both
  match, so the ordering of rules in the file does not matter.
- **The `[risk]` table does not decide anything.** It escalates. `opaque_command` turns a
  command whose effects cannot be read, such as `python cleanup.py`, into a decision for a
  person rather than an allow.

## Try it without an agent

Put an action request in a file and decide it directly. This is the same path the hook
takes, so it is a safe way to test a policy change.

```console
$ cat > /tmp/request.json <<'JSON'
{
  "schema": "shiin.action-request/v1-draft.1",
  "id": "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b",
  "session": "manual",
  "agent": { "id": "claude-code", "assurance": "asserted" },
  "action_type": "fs.read",
  "effects": [
    { "kind": "path", "value": "/home/dev/project/.env", "confidence": "exact" }
  ],
  "workspace": { "root": "/home/dev/project" },
  "policy_snapshot": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "host": { "name": "claude-code", "tool": "Read" }
}
JSON
$ shiin explain /tmp/request.json
decision      Deny
reason        deny.policy
stage         policy
rules         no-secrets
```

Change `value` to a path under `src/` and the same command reports an allow.

## Make one change

Allow the formatter, and nothing more:

```toml
[[rule]]
id = "run-formatter"
effect = "allow"
action_types = ["shell.exec"]
commands = ["cargo fmt*"]
```

Then check the policy still compiles, which also catches a mistyped key:

```console
$ shiin policy validate
.shiin/policy.toml is valid: 4 rule(s), 5 risk signal(s), default Deny
```

## Read next

- [Writing policies](../guides/writing-policies.md), for the full set of keys
- [Policy model](../spec/policy-model.md), for the rules the engine follows
