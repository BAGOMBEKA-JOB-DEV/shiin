<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.2 reviewed=2026-10-12 -->

# Declaring tasks

> [!NOTE]
> **Guide: draft.**
> This works with a source build. The feature is experimental: the constraint set is
> deliberately small, and it will grow based on what turns out to be needed.
> [Intent binding](../spec/intent-binding.md) is the normative source.

Policy says what an agent may ever do in this workspace. A task says what it may do
*right now*, for the work you actually asked for. An action can be refused for not
belonging to the task even when policy would allow it.

## Why this exists

Suppose your policy lets agents write anywhere in the workspace, which is reasonable.
You ask an agent to fix a login bug. It decides the cleanest fix is a database migration
and starts writing one. Policy has no opinion, because the policy was written for the
repository, not for this request.

A task gives Shiin the missing context, and it comes from you rather than from the model.

## Declare one

```console
$ shiin task start "Fix the login bug" \
    --scope './src/auth/**' \
    --scope './tests/auth/**' \
    --forbid db.schema_change \
    --max-writes 20 \
    --minutes 90
```

Quote the globs, so your shell does not expand them before Shiin sees them.

| Option | Effect |
|---|---|
| `--scope <glob>` | Changes outside these paths are refused. Repeatable |
| `--forbid <action type>` | These action types are refused. Repeatable |
| `--allow-host <host>` | Network effects to any other host are refused. `*.example.com` matches subdomains. Repeatable |
| `--ask-for <action type>` | These action types always need a person, even when policy allows them. Repeatable |
| `--max-writes <n>` | Most files the task may write or create |
| `--max-deletes <n>` | Most files the task may delete |
| `--max-commands <n>` | Most commands the task may run |
| `--minutes <n>` | The task ends after this long |

## What it looks like in use

```console
$ shiin task show
id            0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b
description   Fix the login bug
scope         ./src/auth/**, ./tests/auth/**
forbidden     db.schema_change
hosts         none
ask for       none
expires       2026-10-12T15:04:00Z
writes        3 of 20
deletes       0 of none
commands      0 of none
```

A write under `src/auth/` proceeds. A write to `migrations/0042.sql` is refused with
reason code `deny.intent`, and the audit log records which task refused it.

When the work is done:

```console
$ shiin task end
```

## Rules worth knowing

- **A task only ever narrows.** It cannot permit something policy refuses. A denial from
  either one is a denial.
- **Only you can declare a task.** An agent cannot start one for itself. If it could, a
  manipulated agent would simply declare a task that permitted what it wanted.
- **Reads are not scope-limited.** A task that stopped an agent from reading the
  repository would stop it from working. Policy still governs what may be read, which is
  what keeps secrets shut.
- **A refused action does not spend the budget.** Counters move only when an action is
  allowed.
- **An expired task has ended.** Its constraints stop applying, and policy alone decides
  again. Shiin does not keep enforcing yesterday's instructions.
- **Natural language is never enforced.** The description is recorded for the audit log.
  Only the constraints are checked, because only they can be checked exactly.

## Limits of this

A task constrains the actions Shiin sees. It cannot constrain what a program does once
that program is allowed to run, so `python cleanup.py` is still opaque and still needs a
person. See [limitations](../concepts/limitations.md).

Counters live in `.shiin/tasks/`, which policy cannot grant an agent access to
([ADR-0016](../adr/0016-built-in-protected-resources.md)).

## Read next

- [Intent binding](../spec/intent-binding.md)
- [Testing policies](testing-policies.md)
- [Writing policies](writing-policies.md)
