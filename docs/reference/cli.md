<!-- shiin-doc: kind=reference status=draft implementation=partial milestone=v0.4 reviewed=2026-10-16 -->

# CLI

> [!NOTE]
> **Reference: draft.**
> The commands marked "works today" run from a source build.
> Shiin has no releases yet, so there is nothing to install from a package manager.

```console
$ shiin <command>
```

## Commands

| Command | What it does | Status |
|---|---|---|
| `shiin init` | Writes a starter policy to `.shiin/policy.toml`, refusing to overwrite one | works today |
| `shiin policy validate [path]` | Parses and compiles a policy, reporting rule and signal counts or a diagnostic | works today |
| `shiin check <request.json>` | Decides one action request and prints the decision and reason code | works today |
| `shiin explain <request.json>` | Decides, and also prints the stage that decided and the rules that matched | works today |
| `shiin hook claude-code` | Serves one Claude Code `PreToolUse` hook call, reading the payload on standard input | works today |
| `shiin log [count]` | Prints recent audit events, most recent last | works today |
| `shiin verify` | Checks the audit log's hash chain | works today |
| `shiin verify <evidence.json> <public.json>` | Verifies a signed receipt or checkpoint; checkpoints also require matching local audit history | implemented in source; platform validation incomplete |
| `shiin replay <encrypted-replay.json> [policy.toml]` | Replays captured inputs with the original or alternate policy | implemented in source; requires the originating OS keyring |
| `shiin key init`, `shiin key rotate`, `shiin key public` | Initializes or rotates the evidence key and prints its public key | implemented in source on supported keyring platforms |
| `shiin doctor` | Reports this workspace's policy, audit log, mode, and enforcement level | works today |
| `shiin policy test [dir]` | Runs policy cases from `.shiin/tests`, using the same runner as the conformance corpus | works today |
| `shiin task start \| show \| end` | Declares what the agent is working on, shows it with its usage, or ends it | works today |
| `shiin hook codex`, `shiin hook cursor` | The same hook service for those hosts | works today, untested against a running host |
| `shiin pending`, `shiin approve` | Lists and resolves approval requests using a separate operator credential and terminal confirmation | implemented in source on Unix; validation in progress |
| `shiin grants`, `shiin revoke` | Inspects and revokes grants or unresolved tickets | implemented in source on Unix; validation in progress |
| `shiind --operator` | Runs the local approval daemon and displays its in-memory operator credential in a dedicated terminal | implemented in source on Unix; validation in progress |

`shiin policy test` exits non-zero when any case fails, so it belongs in whatever runs your checks.

## Exit codes

`shiin check` and `shiin explain` report the decision as an exit code, so a script can branch on it.

| Code | Meaning |
|---|---|
| 0 | The action is allowed |
| 1 | The action is denied, or the command failed |
| 2 | The action is pending, so a person needs to decide |

`shiin hook claude-code` exits 0 whenever it produced a response, including a denial,
because the host reads the decision from the response rather than from the exit code.
It exits 2 only when it could not produce a response at all, which the host treats as blocking.

## Files it uses

See [configuration](configuration.md). In short, policy is read from `.shiin/policy.toml`
and events are appended to `.shiin/audit.jsonl`.

## Read next

- [Quickstart with Claude Code](../getting-started/quickstart-claude-code.md)
- [Error codes](error-codes.md)
- [Configuration](configuration.md)
