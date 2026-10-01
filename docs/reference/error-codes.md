<!-- shiin-doc: kind=reference status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Error codes

> [!NOTE]
> **Reference: draft.**
> Facts to look up. No implementation exists yet.

Every operational error and diagnostic has a stable code.
A code is never renumbered or reused.

## Policy errors

| Code | Meaning |
|---|---|
| `SHIIN-POL-0001` | The policy file could not be read. |
| `SHIIN-POL-0002` | The policy file is not valid TOML. |
| `SHIIN-POL-0003` | The policy uses an unknown key. |
| `SHIIN-POL-0004` | A rule has a duplicate id. |

## Classification errors

| Code | Meaning |
|---|---|
| `SHIIN-CLS-0001` | A path could not be canonicalized. |

## Audit errors

| Code | Meaning |
|---|---|
| `SHIIN-AUD-0001` | The audit log could not be written. |
| `SHIIN-AUD-0002` | The audit log's hash chain is broken. |

## General errors

| Code | Meaning |
|---|---|
| `SHIIN-GEN-0001` | An internal error occurred. |
| `SHIIN-GEN-0002` | The operation timed out. |

## Read next

- [Error handling](../engineering/error-handling.md)
- [CLI](cli.md)