<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.2 reviewed=2026-10-15 -->

# OpenAI Codex CLI

> [!NOTE]
> **Guide: draft.**
> The adapter exists in a source build. Canonical payload field names were checked against official documentation on 2026-10-15.
> Live-host validation remains open; check the first decision in `shiin log`.

**Enforcement level: L1 cooperative**, and weaker than Claude Code's, for reasons below.
See [enforcement levels](../../security/enforcement-levels.md).

## Configure the hook

Codex runs a `PreToolUse` hook and reads a decision back in the same shape Claude Code
uses (verified against <https://learn.chatgpt.com/docs/hooks> on 2026-09-14). Put this in
`.codex/hooks.json` in the project, or in `~/.codex/hooks.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [{ "type": "command", "command": "shiin hook codex", "timeout": 10 }]
      }
    ]
  }
}
```

Codex requires a hook to be trusted before it runs. Run `/hooks` inside Codex and trust
this one, or it will not be consulted at all.

## What it maps

| Codex tool | Action type | Notes |
|---|---|---|
| `Bash`, `shell`, `local_shell` | `shell.exec` | Classified like any command, so `cat .env` records the read it performs |
| `apply_patch` | `fs.write` | The path is used when the payload names one |
| `Read` | `fs.read` | |
| Anything else | `x-codex.<tool>` | No rule can match it, so it falls through to the default decision |

A patch whose payload names no readable path becomes an **opaque** effect because it still changes files.
The documented `apply_patch` payload carries patch text in `tool_input.command`; the current adapter does not parse that text into file effects.
This remains a classification limitation, independent of payload-field verification.
Canonical `session_id`, `cwd`, `tool_name`, and `tool_input`, including the command field for Bash and patches, were checked against [official hooks documentation](https://learn.chatgpt.com/docs/hooks#pretooluse) (verified 2026-10-15).

## What this does not cover

- **Hosted tools are not intercepted.** Codex's own web search does not reach a hook, so
  Shiin never sees it.
- **Hooks are advisory.** Codex's documentation calls them "a useful guardrail, not a
  complete enforcement boundary" (verified 2026-09-14), and
  `--dangerously-bypass-hook-trust` exists.
- **Codex has its own sandbox and approval policy.** Keep them on. Shiin decides whether
  an action should happen; a sandbox limits the damage when something is allowed and
  misbehaves. See [using Shiin with sandboxes](../using-shiin-with-sandboxes.md).

## Verifying it works

```console
$ shiin log
$ shiin doctor
```

Ask Codex to read a `.env` file. If the log shows no `request.received` event, the hook is
not reaching Shiin: check that it is trusted, and that `shiin` is on the path Codex uses.

If a decision looks wrong, the most likely cause is a payload field this adapter does not
recognize. Please open an integration issue with the payload, secrets removed.

## Read next

- [Claude Code](claude-code.md)
- [Enforcement levels](../../security/enforcement-levels.md)
