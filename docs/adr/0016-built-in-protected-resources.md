<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0016: Built-in protected resources

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Policy model](../spec/policy-model.md), [Security model](../security/security-model.md)

## Decision outcome

A built-in protected resource is a resource that no policy can make allowable for agents.
Protected resources include Shiin's own state, keys, and audit log, host hook configuration, and approval commands.

## Consequences
- Protected resources are hardcoded and cannot be escalated by policy.
- The list is reviewed before each release.
