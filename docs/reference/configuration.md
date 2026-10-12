<!-- shiin-doc: kind=reference status=draft implementation=partial milestone=v0.1 reviewed=2026-10-12 -->

# Configuration

> [!NOTE]
> **Reference: draft.**
> This describes what a source build reads today, and marks what is planned.

## Files

| File | Purpose | Status |
|---|---|---|
| `.shiin/policy.toml` | The policy for this workspace | works today |
| `.shiin/audit.jsonl` | The append-only audit log | works today |
| `.shiin/config.toml` | Settings that are not policy, such as the audit log's location | planned, v0.2 |
| `.shiin/tasks/` | Task declarations | planned, v0.2 |

Paths are relative to the workspace, which is the directory the host reports as its
working directory. Under [ADR-0016](../adr/0016-built-in-protected-resources.md) everything
under `.shiin/` is a built-in protected resource, so no policy can let an agent change it.

A per-user state directory in the usual operating-system locations arrives with the
daemon in v0.3; see [state and storage](../architecture/state-and-storage.md).

## Environment variables

| Variable | Description | Status |
|---|---|---|
| `SHIIN_WORKSPACE` | Overrides the workspace root. Useful when a host reports no working directory | works today |
| `SHIIN_UNATTENDED` | If set to any value, a `pending` decision becomes a denial, because no approver can be reached | works today |
| `CI` | Treated the same as `SHIIN_UNATTENDED`, since continuous integration has no approver | works today |
| `SHIIN_LOG` | Log level filter | planned, v0.2 |

## Options

Global options such as `--config` and `--state` are planned for v0.2, once there is
configuration beyond policy to point at. Today the workspace and the environment
variables above are the only inputs.

## Read next

- [CLI](cli.md)
- [Error codes](error-codes.md)
- [Writing policies](../guides/writing-policies.md)
