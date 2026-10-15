<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-15 -->

# Quickstart with Claude Code

> [!NOTE]
> **Guide: draft.**
> These steps work with a source build. Read [installation](installation.md) first.

By the end you will have refused a real action: Claude Code will ask to read a `.env`
file, and Shiin will say no.

## 1. Write a policy

From the root of the project you want to protect:

```console
$ shiin init
$ shiin policy validate
```

`init` writes `.shiin/policy.toml`. It allows reads and edits under `src/`, `tests/`,
and `docs/`, allows a few test and status commands, denies reads of secrets, and denies
everything it does not mention.

## 2. Connect the hook

Claude Code can run a program before each tool use and read a decision back
(verified against <https://code.claude.com/docs/en/hooks> on 2026-09-14).
Add this to `.claude/settings.json` in the same project:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "shiin hook claude-code",
            "timeout": 10,
            "onFailure": "block"
          }
        ]
      }
    ]
  }
}
```

The matcher `*` sends every tool call to Shiin, which is what you want: a rule that
only covers file tools would miss the same file being read by a shell command.

`onFailure` requires Claude Code 2.1.295 or later and denies the action when the hook cannot start, crashes, or times out
([hooks reference](https://code.claude.com/docs/en/hooks#block-the-action-when-a-hook-fails), verified 2026-10-15).
The internal watchdog covers a running Shiin process; host configuration also covers failures before it starts.

## 3. Watch it refuse something

Start Claude Code in that project and ask it to read your `.env` file.
The hook answers, and the agent is told the action was denied.

Then look at what happened:

```console
$ shiin log
   1  2026-10-12T13:34:39Z  request.received   fs.read
   2  2026-10-12T13:34:39Z  decision.made      deny (deny.policy)
$ shiin verify
the audit log is intact: 2 event(s), chain verified
```

Ask it to run `cat .env` instead. That is a shell command, not a file tool, and it is
denied by the same rule, because the classifier records the read the command performs.

## 4. Know what you have

```console
$ shiin doctor
```

The line that matters most reads `enforcement   L1 cooperative`. It means Claude Code
honors Shiin's decisions but can be started without hooks, and that Shiin never sees an
action the host does not send it. [Enforcement levels](../security/enforcement-levels.md)
explains what that does and does not protect, and
[using Shiin with sandboxes](../guides/using-shiin-with-sandboxes.md) covers the rest.

## Read next

- [Your first policy](first-policy.md)
- [Writing policies](../guides/writing-policies.md)
- [Limitations](../concepts/limitations.md)
