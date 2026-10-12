<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.2 reviewed=2026-10-12 -->

# Testing policies

> [!NOTE]
> **Guide: draft.**
> This works with a source build.

A policy is code: it decides things, and it breaks quietly. `shiin policy test` runs cases
you write, so a change to a rule cannot silently stop protecting something.

The cases use the same format as the project's own
[test vectors](../spec/test-vectors/README.md), and the same runner, so what you write is
checked exactly the way the specification's corpus is.

## Write a case

Put one JSON file per case in `.shiin/tests/`. A case that carries no policy of its own is
evaluated against your workspace's policy, which is what makes it a regression test.

```json
{
  "schema": "shiin.test-vector/v1-draft.1",
  "id": "secrets-stay-shut",
  "name": "No agent reads .env, however it asks",
  "action_request": {
    "schema": "shiin.action-request/v1-draft.1",
    "id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b",
    "session": "manual",
    "agent": { "id": "claude-code", "assurance": "asserted" },
    "action_type": "shell.exec",
    "effects": [
      { "kind": "command", "value": "cat .env", "confidence": "exact" },
      {
        "kind": "path",
        "value": "/home/dev/project/.env",
        "confidence": "inferred",
        "action_type": "fs.read"
      }
    ],
    "workspace": { "root": "/home/dev/project" },
    "policy_snapshot": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "host": { "name": "claude-code", "tool": "Bash" }
  },
  "expected": { "decision": "deny", "reason": "deny.policy" }
}
```

## Run them

```console
$ shiin policy test
ok      secrets-stay-shut  deny.policy
FAILED  bulk-delete-asks  expected pending/pending.risk, got allow/allow.policy

2 case(s), 1 failed
```

The command exits non-zero when any case fails, so it belongs in whatever runs your
checks. Point it elsewhere with `shiin policy test <directory>`.

## What to write cases for

- **Every rule you rely on to refuse something.** A deny rule that stops matching is
  invisible until it matters.
- **The ways around a rule.** The same read through a file tool, through `cat`, and
  through a script. The third is opaque, so expect `pending.opaque` rather than a denial.
- **Each risk signal you configured**, with a case just over the threshold.
- **Your task constraints**, by including a `task` and, for a limit, a `usage` showing what
  has been spent already.
- **Anything you have ever had to fix.** A case is cheaper than remembering.

## Fields a case may carry

| Field | Purpose |
|---|---|
| `action_request` | The action to decide. Required |
| `expected` | The decision and reason code. Required |
| `policy` | A policy for this case alone. Omit to use the workspace's |
| `task` | An active task declaration |
| `usage` | What the task has already spent, for limit cases |
| `mode` | `unattended` to check what happens with no approver |
| `now` | The evaluation time, for expiry cases |

## Read next

- [Writing policies](writing-policies.md)
- [Declaring tasks](declaring-tasks.md)
- [Test vectors](../spec/test-vectors/README.md)
