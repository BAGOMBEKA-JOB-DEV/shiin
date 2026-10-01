<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0007: No telemetry

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Privacy and telemetry](../project/privacy-and-telemetry.md)

## Decision outcome

Shiin collects no telemetry and has no phone-home behavior.
It needs no service and sends no data off the developer's machine.
Every network feature is opt-in.

## Consequences
- The engine performs no networking.
- Audit logs and policy stay on the developer's machine.