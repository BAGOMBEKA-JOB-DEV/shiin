<!-- shiin-doc: kind=explanation status=draft implementation=partial milestone=m0 reviewed=2026-10-16 -->

# Decision pipeline

> [!NOTE]
> **Design document: draft.**
> Authorization, approvals, evidence, and early client/integration code exist in source; platform, host, and milestone validation remain incomplete.

The decision pipeline is the path an [action](../glossary.md#action) takes from proposal to enforcement.

## Steps

1. **Intercept.** An [adapter](../glossary.md#adapter) receives the proposed action from the host.
2. **Normalize.** The adapter builds an [action request](../spec/action-request.md).
3. **Evaluate.** The [engine](../glossary.md#engine) runs five stages in order: identity, capability, policy, intent, and risk.
4. **Decide.** The engine produces a [decision](../spec/decision.md): `allow`, `deny`, or `pending`.
5. **Record.** Request and decision [audit events](../spec/audit-events.md) are synced; a write failure denies the action.
6. **Enforce.** The adapter returns the decision in the host's own protocol.

## Stage order

Later stages can only make the result stricter.
The identity, capability, intent, and risk stages are [gates](../glossary.md#gate): they can stop or hold an action but never allow it.
Only a policy rule, or a [grant](../glossary.md#grant) created by an approver, can produce `allow`.

## Failure

Any failure produces `deny` ([ADR-0013](../adr/0013-fail-closed-failure-semantics.md)).

## Diagram

This diagram shows the Unix daemon path when approval handling is enabled.
The daemon evaluates and audits the request, creates a ticket for `pending`, and lets a separate operator terminal resolve it.
The identity gate currently passes through; the capability gate denies missing or explicitly refused action/effect classes.

```mermaid
sequenceDiagram
    participant Host as Host and agent
    participant Adapter as Host adapter
    participant Daemon as Unix daemon
    participant Approver as Approver
    participant Audit as Audit log
    Host->>Adapter: proposed action (host payload)
    Adapter->>Daemon: action request
    Daemon->>Daemon: identity, capability, policy, intent, risk
    Daemon->>Audit: request received, decision made
    Daemon-->>Adapter: pending ticket
    Adapter->>Approver: ticket shown in operator terminal
    Approver->>Daemon: authenticated resolution
    Daemon->>Audit: approval resolved, grant consumed
    Daemon-->>Adapter: allow
    Adapter-->>Host: allow
```

## Read next

- [How Shiin works](../concepts/how-shiin-works.md)
- [Evaluation](../spec/evaluation.md)
