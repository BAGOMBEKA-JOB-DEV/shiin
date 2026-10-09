<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0033: Enforcement levels

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Enforcement levels](../security/enforcement-levels.md), [Glossary](../glossary.md#enforcement-level), [Glossary](../glossary.md#adapter), [Landscape](../overview/landscape.md), [ADR-0025](0025-adapter-roadmap.md)

## Context and problem statement

Different integration methods provide different strengths of enforcement.
A hook-based adapter can be disabled by the host; a proxy in the data path is harder to bypass; an OS-level control is strongest but hardest to deploy.
The project needed a labeling scheme that honestly communicates how strong each integration is, so operators do not overestimate their protection.

## Decision drivers

- Operators must know how strong their protection is.
- The scheme must be simple and honest.
- Labels must not overpromise.

## Considered options

1. Three labeled levels: L1 cooperative, L2 mediated, L3 OS-enforced.
2. A numeric score (1–10) for each adapter.
3. No labels; let operators infer the strength.

## Decision outcome

Chosen option: "Three labeled levels", because three levels are easy to remember and honest about what each integration can and cannot do.

- **L1 Cooperative:** The host honors the decision through its hook mechanism, which can be disabled (for example, `claude code --bare` or `disableAllHooks`).
- **L2 Mediated:** Shiin sits in the data path, as the MCP proxy does; the agent cannot route around it without disconnecting.
- **L3 OS-enforced** (future): The operating system or a sandbox enforces the decision, regardless of agent behavior.
- Every adapter states its enforcement level and its blind spots on its integration page.

### Consequences

- Good, because operators know exactly how strong their protection is.
- Good, because the labels prevent overstating coverage.
- Bad, because some operators will want a finer-grained score; the three levels are a deliberate simplification.

### Confirmation

- Each integration page under `guides/integrations/` documents its enforcement level.
- The [enforcement levels page](../security/enforcement-levels.md) defines the criteria.

## Pros and cons of the options

### Three labeled levels

- Good, because it is simple and honest.
- Bad, because it does not capture every nuance.

### Numeric score

- Good, because it is granular.
- Bad, because numbers imply precision that does not exist.

### No labels

- Good, because it avoids prescriptive language.
- Bad, because operators overestimate their protection.

## More information

The landscape page documents each host's hook strengths and limits with verification dates. The vision states: "Shiin is designed to add to host controls, not to replace them."
