<!-- shiin-doc: kind=spec status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# Threat model

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.

This page defines the threats Shiin is designed to resist.
Threats are referenced throughout the documentation as `**[ST-<category>-<nn>]**`.

## STRIDE categories

| Category | Meaning |
|---|---|
| S | Spoofing identity |
| T | Tampering with data or logs |
| R | Repudiation (denying an action) |
| I | Information disclosure |
| D | Denial of service |
| E | Elevation of privilege |

## Threats

### Spoofing

**[ST-S-01]** An agent presents a false identity.
Shiin treats the agent as untrusted and never allows an action because the agent says it is safe.

**[ST-S-02]** An approver is impersonated.
Approver identity is established through an approval channel that the agent does not control.

### Tampering

**[ST-T-01]** An agent modifies the audit log.
The hash chain makes removal or alteration detectable.

**[ST-T-02]** An agent modifies policy.
Policy is loaded from trusted layers; a workspace layer can only restrict.

**[ST-T-03]** An agent modifies Shiin's own state.
Shiin's state is a built-in protected resource that no policy can make allowable.

### Repudiation

**[ST-R-01]** An agent denies it performed an action.
Every decision is recorded in the audit log with a stable id.

### Information disclosure

**[ST-I-01]** An agent learns policy structure to probe for a bypass.
Agent-facing messages never reveal policy structure, error codes, or paths.

**[ST-I-02]** An agent reads secret values through logs.
Logs never contain secret values; identifiers and digests are used instead.

**[ST-I-03]** An agent reads the audit log.
The audit log is a built-in protected resource.

### Denial of service

**[ST-D-01]** An agent floods Shiin with requests.
The engine is synchronous and bounded; the adapter enforces timeouts that produce `deny`.

**[ST-D-02]** A malicious policy file hangs Shiin.
Policy compilation is bounded and fails closed.

### Elevation of privilege

**[ST-E-01]** An agent uses a grant beyond its scope.
A grant is bound to a digest or a scope and never overrides `deny`.

**[ST-E-02]** An agent disables Shiin.
Adapters declare their enforcement level honestly; hook-based controls can be disabled.

**[ST-E-03]** An agent runs code that bypasses Shiin.
Shiin cannot see what an allowed command does; a sandbox covers that gap.

## Out of scope

Shiin is not designed to resist:
- Operating-system level attacks against the developer's account.
- Hardware or firmware attacks.
- Supply-chain attacks against Shiin's dependencies, which are covered by the supply-chain policy.