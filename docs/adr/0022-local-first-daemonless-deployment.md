<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0022: Local-first, daemonless deployment

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [Deployment models](../architecture/deployment-models.md), [ADR-0007](0007-no-telemetry.md)

## Decision outcome

Shiin runs on the developer's machine, needs no service, and collects no telemetry.
In v0.1 the adapter runs as a process for each hook invocation.
A per-user daemon is planned for v0.3.

## Consequences
- No service to install or maintain.
- Fast process start-up on the hook path.
