<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Evaluation

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.evaluation/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

The [engine](../glossary.md#engine) evaluates an [action request](action-request.md) against a [policy snapshot](../glossary.md#policy-snapshot) and produces a [decision](decision.md).
The engine performs no I/O and is deterministic: the same inputs always produce the same decision ([ADR-0009](../adr/0009-deterministic-enforcement.md), [ADR-0010](../adr/0010-pure-synchronous-engine.md)).

## Stages

**[EVAL-001]** The engine MUST run exactly five stages in this fixed order: identity, capability, policy, intent, and risk ([ADR-0012](../adr/0012-evaluation-stage-order.md)).

**[EVAL-002]** Each stage returns one of `allow`, `deny`, `pending`, or `pass`.

**[EVAL-003]** A stage that returns `allow` is a *policy stage*; only the policy stage may produce `allow`.

**[EVAL-004]** The identity, capability, intent, and risk stages are [gates](../glossary.md#gate): they may return `deny`, `pending`, or `pass`, but never `allow`.

## Combining

**[EVAL-005]** If any stage returns `deny`, the decision is `deny` ([ADR-0011](../adr/0011-decision-values-and-combining.md)).

**[EVAL-006]** Otherwise, if any stage returns `pending`, the decision is `pending`, unless a matching [grant](../glossary.md#grant) already covers the action.

**[EVAL-007]** Otherwise, if the policy stage allows the action, the decision is `allow`.

**[EVAL-008]** Otherwise, the decision is the [default decision](../glossary.md#default-decision), which is `deny`.

## Fail-closed

**[EVAL-009]** Any failure inside evaluation MUST produce `deny` with reason code `deny.failure` ([ADR-0013](../adr/0013-fail-closed-failure-semantics.md)).

## Trace

**[EVAL-010]** In explain mode, the engine MUST return a [trace](../glossary.md#trace) recording each stage's result.

**[EVAL-011]** A trace MUST include the stage name, the result, and the reason code or rule id that produced it.

## Policy snapshot

**[EVAL-012]** The engine MUST evaluate against a single immutable [policy snapshot](../glossary.md#policy-snapshot) identified by a digest.

**[EVAL-013]** The snapshot digest MUST be recorded in the action request and the audit event.

## Examples

<!-- validate: spec/schemas/evaluation.v1.schema.json -->
```json
{
  "schema": "shiin.evaluation/v1-draft.1",
  "request_id": "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b",
  "decision": "allow",
  "reason": "allow.policy",
  "trace": [
    { "stage": "identity", "result": "pass", "reason": "identity.asserted" },
    { "stage": "capability", "result": "pass", "reason": "capability.fs_write" },
    { "stage": "policy", "result": "allow", "reason": "rule:edit-source" },
    { "stage": "intent", "result": "pass", "reason": "intent.no_task" },
    { "stage": "risk", "result": "pass", "reason": "risk.none" }
  ]
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.