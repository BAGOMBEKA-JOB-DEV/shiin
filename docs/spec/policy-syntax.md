<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Policy syntax

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.policy-syntax/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

Policy is written in TOML and compiled to Cedar ([ADR-0014](../adr/0014-cedar-policy-engine-with-toml-front-end.md)).
This page defines the TOML front-end.

## Top level

**[SYN-001]** A policy file MUST be a TOML document with a top-level `default` key.

**[SYN-002]** `default` MUST be `deny` or `pending`.

**[SYN-003]** A policy file MAY contain a top-level `capabilities` table.

## Rules

**[SYN-004]** Rules MUST be array elements of a top-level `rules` array.

**[SYN-005]** Each rule MUST be a table with `id` and `effect` keys.

**[SYN-006]** `id` MUST be a unique string within the file.

**[SYN-007]** `effect` MUST be `allow`, `deny`, or `pending`.

**[SYN-008]** A rule MAY match on `action_types`, `resources`, `commands`, `agents`, `sessions`, and `tasks`.

**[SYN-009]** `resources` MUST be glob patterns relative to the workspace root.

**[SYN-010]** `commands` MUST be exact command strings or glob patterns.

## Capabilities

**[SYN-011]** A capability MUST be a table with `id` and either `allow` or `deny`.

**[SYN-012]** A capability MAY match on `action_types` and `agents`.

## Risk configuration

**[SYN-013]** Risk signals are configured with a top-level `risk` table.

**[SYN-014]** Each signal MAY be set to `allow`, `deny`, or `pending`.

## Example

<!-- validate: spec/schemas/policy-syntax.v1.schema.json -->
```json
{
  "schema": "shiin.policy-syntax/v1-draft.1",
  "default": "deny",
  "rules": [
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
    "bulk_delete": "pending",
    "secret_path": "deny"
  }
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.