<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0015: Policy layering and trust

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Policy model](../spec/policy-model.md), [Glossary](../glossary.md#policy-layer), [ADR-0014](0014-cedar-policy-engine-with-toml-front-end.md)

## Context and problem statement

Policy comes from multiple sources: built-in defaults, the operator, the workspace, and eventually an organization.
These sources have different trust levels: the operator's policy is more trusted than a workspace's.
The project needed a layering model that allows workspace policy to only restrict, never to grant new allowances, unless the operator explicitly trusts it.

## Decision drivers

- Built-in rules (forbid reading secrets, forbid modifying Shiin state) must always apply.
- Operator policy must take precedence over workspace policy.
- Workspace policy must be able to deny or narrow, but not broaden, what the operator allows.
- The layering must be explicit and auditable.

## Considered options

1. Three layers (built-in, user, workspace) where later layers can only restrict.
2. A flat policy where all sources merge into one set of rules.
3. A plugin system where each layer is loaded by a different trusted party.

## Decision outcome

Chosen option: "Three layers where later layers can only restrict", because it gives operators control without requiring per-workspace review.

- Layers, in order: built-in, user, and workspace. An organization layer is reserved for the future.
- Built-in rules define protected resources and the default decision.
- User (operator) rules are loaded from the operator's configuration.
- Workspace rules are loaded from the project directory and can only restrict unless the operator trusts the workspace by hash.
- A workspace layer can only restrict unless the operator trusts it by hash.

### Consequences

- Good, because an untrusted workspace cannot grant access the operator denied.
- Good, because the layers are explicit and auditable.
- Bad, because operators who want workspace policy to grant new allowances must explicitly trust each workspace.

### Confirmation

- The [policy model](../spec/policy-model.md) specifies the layering and the trust rule.
- The [built-in protected resources specification](../spec/policy-model.md) defines what built-in rules cover.

## Pros and cons of the options

### Three layers, workspace can only restrict

- Good, because it is safe by default.
- Bad, because it requires an explicit trust step for full workspace flexibility.

### Flat merged policy

- Good, because it is simple.
- Bad, because a malicious or careless workspace can override operator policy.

### Plugin system per layer

- Good, because it is flexible.
- Bad, because it adds complexity and a trusted-loader component.

## More information

The [built-in protected resources ADR](0016-built-in-protected-resources.md) defines what built-in rules cover. The policy model specification defines the evaluation order of layers.
