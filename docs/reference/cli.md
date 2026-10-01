<!-- shiin-doc: kind=reference status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# CLI

> [!NOTE]
> **Reference: draft.**
> Facts to look up. No implementation exists yet.

The `shiin` command-line binary provides these subcommands.

## Commands by milestone

| Command | Description | Milestone |
|---|---|---|
| `shiin init` | Create a workspace policy file. | v0.1 |
| `shiin check` | Check that a policy file is valid. | v0.1 |
| `shiin explain` | Explain a decision in detail. | v0.1 |
| `shiin policy validate` | Validate a policy and report diagnostics. | v0.1 |
| `shiin policy test` | Test a policy against recorded cases. | v0.2 |
| `shiin hook claude-code` | Run the Claude Code hook adapter. | v0.1 |
| `shiin hook codex` | Run the Codex CLI hook adapter. | v0.2 |
| `shiin hook cursor` | Run the Cursor hook adapter. | v0.2 |
| `shiin task` | Manage task declarations. | v0.2 |
| `shiin pending` | Manage pending approvals. | v0.3 |
| `shiin approve` | Resolve an approval request. | v0.3 |
| `shiin log` | Show the audit log. | v0.1 |
| `shiin verify` | Verify the audit log's hash chain. | v0.4 |
| `shiin replay` | Re-evaluate recorded requests. | v0.4 |
| `shiin doctor` | Diagnose the Shiin installation. | v0.1 |

## Global options

| Option | Description |
|---|---|
| `--config <path>` | Path to a Shiin configuration file. |
| `--state <path>` | Path to Shiin's state directory. |
| `--json` | Emit machine-readable output. |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. |
| 1 | A user-facing error, such as an invalid policy. |
| 2 | An internal error, which produces a fail-closed deny at run time. |

## Read next

- [Configuration](configuration.md)
- [Error codes](error-codes.md)
- [Platform support](platform-support.md)