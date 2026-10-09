<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0016: Built-in protected resources

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Policy model](../spec/policy-model.md), [Glossary](../glossary.md#built-in-protected-resource), [ADR-0015](0015-policy-layering-and-trust.md)

## Context and problem statement

An agent might try to modify Shiin's own configuration, delete its audit log, or exfiltrate its signing keys.
If policy can allow these actions, an operator's mistake or a compromised workspace policy could disable Shiin entirely.
The project needed a mechanism that makes certain resources unallowable by any policy.

## Decision drivers

- Shiin's state, keys, and audit log must not be modifiable by any agent.
- Host hook configuration that disables interception must be protected.
- Approval commands must not be callable by an agent.
- These protections must be in the built-in layer and cannot be overridden.

## Considered options

1. Built-in rules that deny access to protected resources, unallowable by any policy layer.
2. No built-in rules; rely entirely on operator policy.
3. OS-level file permissions as the sole protection.

## Decision outcome

Chosen option: "Built-in rules that deny access to protected resources", because policy that can be overridden is not a guarantee.

- Built-in protected resources include: Shiin's own state directory, signing keys, audit log, host hook configuration files, and approval commands.
- These resources are unallowable: no policy, no layer, and no grant can permit an action that accesses them.
- The built-in layer is evaluated first and produces `deny` immediately for any matching resource.

### Consequences

- Good, because Shiin cannot disable itself through its own policy system.
- Good, because the protection is automatic and does not require operator configuration.
- Bad, because if the built-in resource list is wrong, legitimate actions are blocked; the list must be carefully maintained.

### Confirmation

- The [policy model](../spec/policy-model.md) lists the built-in resource patterns.
- The glossary defines [built-in protected resources](../glossary.md#built-in-protected-resource).

## Pros and cons of the options

### Built-in unallowable rules

- Good, because the protection is absolute within Shiin's authority.
- Bad, because the resource list must be complete and correct.

### Operator policy only

- Good, because it is flexible.
- Bad, because a misconfiguration can disable Shiin.

### OS-level permissions only

- Good, because the OS enforces them.
- Bad, because OS permissions are coarser and do not understand action effects.

## More information

See [ADR-0015](0015-policy-layering-and-trust.md) for the layering model that places built-in rules at the bottom. The [threat model](../security/threat-model.md) treats tampering with audit logs as an in-scope threat.
