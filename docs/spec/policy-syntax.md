<!-- shiin-doc: kind=spec status=draft implementation=partial milestone=v0.1 reviewed=2026-10-16 -->

# Policy syntax

> [!NOTE]
> **Specification: draft.**
> The source build implements the format below.
> Capabilities and session/task rule matchers are implemented in source; identity assurance and separately trusted layers remain open.
>
> Version: `shiin.policy-syntax/v1-draft.3`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

Policy is written in TOML and compiled to Cedar ([ADR-0014](../adr/0014-cedar-policy-engine-with-toml-front-end.md)).
This page defines the TOML front-end.

## Top level

**[SYN-001]** A policy file MUST be a TOML document with top-level `version = 1`.

**[SYN-002]** `[defaults].decision` MUST be `deny` or `pending`, and MUST default to `deny` when the table is omitted.

**[SYN-003]** A policy file MAY contain a top-level `capabilities` table.

The table maps capability identifiers to declarations; each declaration's `id` equals its table key.
Omitting the table grants no action classes.

## Rules

**[SYN-004]** Rules MUST be array elements of a top-level `rule` array, written as `[[rule]]` in TOML.

**[SYN-005]** Each rule MUST be a table with `id` and `effect` keys.

**[SYN-006]** `id` MUST be a unique string within the file.

**[SYN-007]** `effect` MUST be `allow`, `deny`, or `pending`.

**[SYN-008]** A rule MAY match on `action_types`, `resources`, `commands`, `agents`, `sessions`, and `tasks`.

Session and task selectors are exact identifier lists.
Every nonempty selector must match; a task selector does not match an absent active task.
The daemon replaces caller-supplied task claims with its active workspace task before evaluating these selectors.

**[SYN-009]** Resource globs beginning with `./` MUST resolve against the canonical workspace root.

Absolute globs and operator-home globs beginning with `~/` are also supported.

**[SYN-010]** `commands` MUST be exact command strings or glob patterns.

## Capabilities

**[SYN-011]** A capability MUST be a table with `id` and either `allow` or `deny`.

**[SYN-012]** A capability MAY match on `action_types` and `agents`.

The `allow` or `deny` value is exactly `true`; false, absent, or simultaneous effects are rejected.
`action_types` is a nonempty list using exact classes or a trailing `*` wildcard.
Every matching denial wins, and every request/effect class needs an explicit matching grant.
Empty agent selectors apply to every asserted identifier; they provide no attestation.

## Risk configuration

**[SYN-013]** Risk signals are configured with a top-level `risk` table.

**[SYN-014]** Each signal MAY be set to `allow`, `deny`, or `pending`.

`allow` means the risk gate passes; it never authorizes an action.
A signal may instead be a table with `decision` and an optional positive `threshold`.
The `bulk_delete` default threshold is 20.

**[SYN-015]** Unknown policy keys, unknown risk signal names, and unknown detailed risk settings MUST be rejected.

## Example

<!-- validate: spec/schemas/policy-syntax.v1.schema.json -->
```json
{
  "version": 1,
  "capabilities": {"workspace": {"id": "workspace", "allow": true, "action_types": ["fs.*", "shell.exec"]}},
  "defaults": { "decision": "deny" },
  "rule": [
    {
      "id": "edit-source",
      "effect": "allow",
      "action_types": ["fs.read", "fs.write", "fs.create"],
      "resources": ["./src/**", "./tests/**"]
    },
    {
      "id": "no-secrets",
      "effect": "deny",
      "action_types": ["fs.read"],
      "resources": ["**/.env", "**/.env.*", "~/.ssh/**"],
      "reason": "Secret files are not readable by agents."
    }
  ],
  "risk": {
    "bulk_delete": { "decision": "pending", "threshold": 20 },
    "secret_path": "deny"
  }
}
```

## Revision history

- v1-draft.3 (2026-10-16): Implemented explicit capability tables and session/task selectors, including validation and denial when a class lacks a grant.
- v1-draft.2 (2026-10-15): Reconciled the draft with the implemented TOML front-end and its JSON representation; documented unsupported keys and fail-closed risk configuration.
- v1-draft.1 (2026-10-01): Initial draft.
