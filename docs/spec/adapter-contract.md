<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Adapter contract

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.adapter-contract/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

An [adapter](../glossary.md#adapter) connects a [host](../glossary.md#host) to Shiin.
An adapter intercepts proposed actions, builds [action requests](action-request.md), and enforces the resulting [decision](decision.md) using the host's own protocol.
Adapters never make decisions ([ADR-0025](../adr/0025-adapter-roadmap.md), [ADR-0026](../adr/0026-adapter-extensibility.md)).

## Interception

**[ADP-001]** An adapter MUST intercept proposed actions before the host executes them.

**[ADP-002]** An adapter MUST normalize the host payload into an action request ([AR-001](action-request.md#fields)).

**[ADP-003]** An adapter MUST NOT classify an action more permissively than the host payload allows.
Anything the adapter cannot classify MUST become an [opaque effect](../glossary.md#opaque-effect) ([ADR-0018](../adr/0018-conservative-action-classification.md)).

## Enforcement

**[ADP-004]** An adapter MUST enforce the decision using the host's own protocol.

**[ADP-005]** An adapter MUST return `allow` only when the decision is `allow`.

**[ADP-006]** An adapter MUST return `deny` when the decision is `deny`.

**[ADP-007]** For a `pending` decision, an adapter MUST either wait for an approver, use the host's native prompt, or deny with a ticket ([ADR-0028](../adr/0028-deny-with-ticket-for-non-waiting-hosts.md)).

**[ADP-008]** An adapter MUST produce `deny` on any error, timeout, or crash ([ADR-0013](../adr/0013-fail-closed-failure-semantics.md)).

## Enforcement level

**[ADP-009]** Every adapter MUST declare an [enforcement level](../glossary.md#enforcement-level): L1 cooperative, L2 mediated, or L3 OS-enforced ([ADR-0033](../adr/0033-enforcement-levels.md)).

**[ADP-010]** An adapter's documentation MUST state what it cannot see.

## Audit

**[ADP-011]** An adapter MUST write an audit event before a non-read action is allowed to proceed.

**[ADP-012]** An adapter MUST record every decision, approval, and grant.

## Hosts

| Host | Enforcement level | Interception | Milestone |
|---|---|---|---|
| Claude Code | L1 | hooks | v0.1 |
| Codex CLI | L1 | hooks | v0.2 |
| Cursor | L1 | hooks | v0.2 |
| MCP proxy | L2 | data path | v0.5 |

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.