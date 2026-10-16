<!-- shiin-doc: kind=reference status=draft implementation=partial milestone=v0.4 reviewed=2026-10-16 -->

# Configuration

> [!NOTE]
> **Reference: draft.**
> This describes what a source build reads today, and marks what is planned.

## Files

| File | Purpose | Status |
|---|---|---|
| `.shiin/policy.toml` | The policy for this workspace | works today |
| `.shiin/audit.jsonl` | The append-only audit log | works today |
| `.shiin/tests/` | Policy cases run by `shiin policy test` | works today |
| `.shiin/config.toml` | Settings that are not policy, such as the audit log's location | planned, v0.3 |
| `.shiin/tasks/` | The active task declaration and its counters | works today |

Paths are relative to the workspace, which is the directory the host reports as its
working directory. Under [ADR-0016](../adr/0016-built-in-protected-resources.md) everything
under `.shiin/` is a built-in protected resource, so no policy can let an agent change it.

The Unix daemon uses a private per-user runtime directory.
Its default socket and approval state, plus workspace evidence paths, are listed in [State and storage](../architecture/state-and-storage.md).

## Environment variables

| Variable | Description | Status |
|---|---|---|
| `SHIIN_WORKSPACE` | Overrides the workspace root. Useful when a host reports no working directory | works today |
| `SHIIN_UNATTENDED` | If set to any value, a `pending` decision becomes a denial, because no approver can be reached | works today |
| `CI` | Treated the same as `SHIIN_UNATTENDED`, since continuous integration has no approver | works today |
| `SHIIN_DAEMON` | Set to `1` to require local daemon evaluation; an unavailable configured service denies the action | source implementation on Unix |
| `SHIIN_SOCKET` | Absolute Unix socket override; setting it requires the daemon even without `SHIIN_DAEMON=1` | source implementation on Unix |
| `SHIIN_EVIDENCE` | Set to `1` when starting the daemon to enable signed receipts/checkpoints and encrypted replay records; unavailable keys prevent startup | source implementation; OS key-store validation remains open |
| `SHIIN_LOG` | Log level filter | planned, v0.2 |

## Options

Global options such as `--config` and `--state` are planned for v0.2, once there is
configuration beyond policy to point at. Today the workspace and the environment
variables above are the only inputs.

## Read next

- [CLI](cli.md)
- [Error codes](error-codes.md)
- [Writing policies](../guides/writing-policies.md)
