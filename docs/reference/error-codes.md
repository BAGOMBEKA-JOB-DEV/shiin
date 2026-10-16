<!-- shiin-doc: kind=reference status=draft implementation=partial milestone=v0.1 reviewed=2026-10-16 -->

# Error codes

> [!NOTE]
> **Reference: draft.**
> Every code marked "emitted" is produced by the current source build and is checked
> against the source by `cargo xtask docs-check`.

A code is stable: it is never renumbered or reused, so a script may match on it.
A code names an operational problem, never a decision.
A denial is not an error, and it carries a [reason code](../spec/decision.md) instead.

## Policy errors

| Code | Meaning | Status |
|---|---|---|
| `SHIIN-POL-0001` | The policy file could not be read. | emitted |
| `SHIIN-POL-0002` | The policy file is not valid TOML. | emitted |
| `SHIIN-POL-0003` | The policy uses an unknown key. | emitted |
| `SHIIN-POL-0004` | A rule has a duplicate id. | emitted |
| `SHIIN-POL-0005` | A rule has an invalid glob pattern. | emitted |
| `SHIIN-POL-0006` | A rule could not be compiled for the policy engine. | emitted |

Every one of these results in a denial, because a policy that cannot be understood
cannot authorize anything ([ADR-0013](../adr/0013-fail-closed-failure-semantics.md)).

## Classification errors

| Code | Meaning | Status |
|---|---|---|
| `SHIIN-CLS-0001` | A path could not be canonicalized. | reserved |

Canonicalization currently cannot fail: a path that does not exist is still resolved,
so the classifier returns a best effort rather than an error. The code is reserved for
the platform-specific cases that Windows support will introduce in v0.6.

## Audit errors

| Code | Meaning | Status |
|---|---|---|
| `SHIIN-AUD-0001` | The audit log could not be written. | emitted |
| `SHIIN-AUD-0002` | The audit log's hash chain is broken, or a line is malformed. | emitted |

`SHIIN-AUD-0001` denies the action it was recording, because an action that cannot be
recorded is not allowed to proceed ([ADR-0030](../adr/0030-audit-durability.md)).

## General errors

| Code | Meaning | Status |
|---|---|---|
| `SHIIN-GEN-0001` | An internal error occurred, including a request that could not be evaluated. | emitted |
| `SHIIN-GEN-0002` | The operation timed out. | reserved |

`SHIIN-GEN-0002` remains reserved; current hook and daemon timeout paths return a fixed denial or transport error rather than emitting that stable code.
Hook evaluation has a two-second deadline, and a local daemon exchange has a 1.5-second deadline.
Neither timeout falls back to an allow decision.

## Read next

- [Error handling](../engineering/error-handling.md)
- [CLI](cli.md)
