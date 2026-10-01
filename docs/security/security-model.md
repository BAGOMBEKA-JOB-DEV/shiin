<!-- shiin-doc: kind=spec status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# Security model

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

## Fail-closed

**[SEC-001]** Any failure, error, timeout, or unreadable policy MUST produce `deny` ([ADR-0013](../adr/0013-fail-closed-failure-semantics.md)).

**[SEC-002]** A failure that happens after a decision but before the adapter answers MUST also produce `deny`.

## Deny wins

**[SEC-003]** A `deny` from any stage overrides every other result, including [grants](../glossary.md#grant).

**[SEC-004]** When no rule matches, the decision is `deny`.

## The agent is untrusted

**[SEC-005]** Anything the agent or a tool server supplies can make a decision stricter but never more permissive.

**[SEC-006]** Agent-supplied identity claims, tool annotations, and explanations are treated as untrusted input.

## Gates

**[SEC-007]** The identity, capability, intent, and risk stages are [gates](../glossary.md#gate): they may return `deny`, `pending`, or `pass`, but never `allow`.

**[SEC-008]** Only a policy rule or a grant may produce `allow`.

## Built-in protected resources

**[SEC-009]** A [built-in protected resource](../glossary.md#built-in-protected-resource) MUST never be made allowable for agents by policy ([ADR-0016](../adr/0016-built-in-protected-resources.md)).

**[SEC-010]** Protected resources include Shiin's own state, keys, and audit log, host hook configuration, and approval commands.

## Determinism

**[SEC-011]** The same action request and the same policy snapshot always produce the same decision.
No language-model or machine-learning output is an input to any decision ([ADR-0009](../adr/0009-deterministic-enforcement.md)).

## Honest coverage

**[SEC-012]** Every adapter states its [enforcement level](../glossary.md#enforcement-level) and what it cannot see ([ADR-0033](../adr/0033-enforcement-levels.md)).

## Sandboxes

**[SEC-013]** Shiin decides whether an action should happen.
A sandbox limits the damage if an allowed action, or the code it runs, misbehaves.

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.