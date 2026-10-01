<!-- shiin-doc: kind=spec status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# Enforcement levels

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

An [enforcement level](../glossary.md#enforcement-level) states how strongly a Shiin decision is enforced for a given [adapter](../glossary.md#adapter) ([ADR-0033](../adr/0033-enforcement-levels.md)).

## Levels

### L1 Cooperative

**[ENF-001]** The host honors the decision through its hook mechanism.

**[ENF-002]** The hook can be disabled with a host flag such as `--bare` or `disableAllHooks`.

**[ENF-003]** An L1 adapter cannot see actions that do not pass through the hook.

### L2 Mediated

**[ENF-004]** Shiin sits in the data path, as the MCP proxy does.

**[ENF-005]** The host must route traffic through Shiin for the decision to apply.

### L3 OS-enforced

**[ENF-006]** The operating system or a sandbox enforces the decision.

**[ENF-007]** L3 is a future enforcement level.

## Per-adapter levels

| Adapter | Level | Cannot see |
|---|---|---|
| Claude Code | L1 | Actions outside hooks; code run by an allowed command. |
| Codex CLI | L1 | Actions outside hooks; code run by an allowed command. |
| Cursor | L1 | Actions outside hooks; code run by an allowed command. |
| MCP proxy | L2 | Traffic not routed through the proxy. |
| Shell wrapper | L1 | Commands typed directly at the shell. |

## Honest labeling

**[ENF-008]** Every adapter's documentation MUST state its enforcement level and what it cannot see.

**[ENF-009]** An adapter MUST NOT claim protection that its interception point cannot provide.

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.