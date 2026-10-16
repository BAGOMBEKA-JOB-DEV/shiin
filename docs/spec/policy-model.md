<!-- shiin-doc: kind=spec status=accepted implementation=partial milestone=v0.1 reviewed=2026-10-16 -->

# Policy model

> [!NOTE]
> **Specification: accepted.**
> Implemented in part in the source build. Remaining requirements and external validation are not complete.
>
> Version: `shiin.policy-model/v1-draft.2`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

A [policy](../glossary.md#policy) is the complete set of [rules](../glossary.md#rule), [capabilities](../glossary.md#capability), defaults, and risk configuration that applies to an evaluation ([ADR-0015](../adr/0015-policy-layering-and-trust.md)).

The source build implements explicit capabilities, rules, defaults, built-in protections, and configured risk signals.
Separately loaded trusted layers and complete snapshot persistence are not implemented.
The Unix approval service applies grants after fresh evaluation; a grant cannot override a denial.
The CLI currently loads one operator-controlled `.shiin/policy.toml` file; it does not implement workspace trust-by-hash.

## Rules

**[POL-001]** A rule MUST match an action request and produce `allow`, `deny`, or `pending`.

**[POL-002]** A rule MUST have a unique id within its policy layer.

**[POL-003]** A rule MUST specify an effect: `allow`, `deny`, or `pending`.

**[POL-004]** A rule MAY match on `action_type`, `resources`, `commands`, `agents`, `sessions`, and `tasks`.

**[POL-005]** A rule that matches no action request MUST have no effect.

## Capabilities

**[POL-006]** A capability is a coarse permission for an agent to attempt a whole class of actions at all.

**[POL-007]** A capability MUST be granted or denied explicitly; there is no implicit capability.

The capability gate checks the request's action class and every distinct effect action class.
An absent grant or any matching capability refusal produces `deny.capability`; a capability grant only passes this gate and never authorizes an action by itself.
Agent selectors use the adapter's asserted identifier; they do not establish identity assurance.
The snapshot format is now `shiin.policy/v1-draft.2`.
Older snapshots and replay-engine versions are refused rather than silently receiving implicit capabilities.

## Default decision

**[POL-008]** The [default decision](../glossary.md#default-decision) is `deny`.

**[POL-009]** The default decision MAY be configured to `pending`, but it can never be `allow`.

## Built-in protected resources

**[POL-010]** A [built-in protected resource](../glossary.md#built-in-protected-resource) MUST never be made allowable for agents by policy ([ADR-0016](../adr/0016-built-in-protected-resources.md)).

**[POL-011]** Built-in protected resources include Shiin's own state, keys, and audit log, host hook configuration, and approval commands.

## Policy layers

**[POL-012]** Policy is drawn from layers, in order: built-in, user, and workspace ([ADR-0015](../adr/0015-policy-layering-and-trust.md)).

**[POL-013]** A workspace layer can restrict a user layer only; it can never make a deny into an allow unless the operator trusts it by hash.

**[POL-014]** An organization layer is reserved for the future.

## Policy snapshot

**[POL-015]** A [policy snapshot](../glossary.md#policy-snapshot) is the immutable input to one evaluation: compiled policy, grants, active tasks, and counters.

**[POL-016]** A policy snapshot MUST be identified by a digest recorded in the action request and the audit event.

## Examples

<!-- validate: spec/schemas/policy.v1.schema.json -->
```json
{
  "schema": "shiin.policy/v1-draft.2",
  "layers": [
    {
      "name": "built-in",
      "digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
      "rules": [
        {
          "id": "protect-audit-log",
          "effect": "deny",
          "action_types": ["fs.write"],
          "resources": [".shiin/audit.jsonl"],
          "reason": "The audit log is a protected resource."
        }
      ]
    }
  ],
  "default": "deny"
}
```

## Revision history

- v1-draft.2 (2026-10-16): Added explicit capability declarations, default-deny capability enforcement, and session/task rule selectors; prior snapshots require migration by the operator.
- Accepted on 2026-10-12. Accepted: the spike compiled policies in this model to Cedar and reproduced every expected decision.
- v1-draft.1 (2026-10-01): Initial draft.
