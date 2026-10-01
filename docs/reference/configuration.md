<!-- shiin-doc: kind=reference status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Configuration

> [!NOTE]
> **Reference: draft.**
> Facts to look up. No implementation exists yet.

## Configuration files

| File | Purpose | Location |
|---|---|---|
| `policy.toml` | Policy rules and capabilities | `.shiin/policy.toml` |
| `config.toml` | Shiin configuration | `.shiin/config.toml` |
| `tasks/` | Task declarations | `.shiin/tasks/` |

## Global options

| Option | Description |
|---|---|
| `--config <path>` | Path to a Shiin configuration file. |
| `--state <path>` | Path to Shiin's state directory. |

## Environment variables

| Variable | Description |
|---|---|
| `SHIIN_STATE` | Shiin's state directory. Overrides the default. |
| `SHIIN_LOG` | Log level filter, such as `shiin=info`. |

## Read next

- [CLI](cli.md)
- [Error codes](error-codes.md)