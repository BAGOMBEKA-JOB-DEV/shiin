<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0022: Local-first, daemonless deployment

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Vision](../overview/vision.md#principles), [Glossary](../glossary.md#engine), [ADR-0007](0007-no-telemetry.md), [ADR-0023](0023-json-rpc-local-api.md), [ADR-0010](0010-pure-synchronous-engine.md)

## Context and problem statement

Shiin intercepts agent actions on the developer's machine.
The v0.1 adapter runs as a Claude Code hook: a new process for each invocation, with no long-lived daemon.
This must remain the zero-dependency path: no service, no network, no daemon, no telemetry.
The daemon (v0.3) is an addition for approvals, not a requirement.

## Decision drivers

- The hook path must start fast, because it runs on every action.
- No service should be required for basic authorization.
- No data should leave the machine.

## Considered options

1. Daemonless hooks for v0.1, with an optional daemon added later for approvals.
2. A daemon from v0.1 that all hooks connect to.
3. A cloud service that all agents connect to.

## Decision outcome

Chosen option: "Daemonless hooks for v0.1, optional daemon later", because the daemonless path has lower latency and zero setup cost.

- v0.1 runs a new `shiin hook` process for each host invocation.
- The engine is pure and synchronous ([ADR-0010](0010-pure-synchronous-engine.md)), so a hook process can evaluate and exit.
- The daemon (`shiind`) is introduced in v0.3 for approvals and the local API, but is optional.
- The daemonless path remains supported after the daemon ships.

### Consequences

- Good, because the barrier to entry is zero: no daemon to start or configure.
- Good, because each hook invocation is independent and fast.
- Bad, because approvals in v0.1 use the host's native prompt; the real approval protocol comes in v0.3.

### Confirmation

- The [roadmap](../project/roadmap.md) lists `shiind` and the approval protocol in v0.3.
- The [use cases](../overview/use-cases.md) for unattended CI (UC-07) require deterministic behavior without a daemon.

## Pros and cons of the options

### Daemonless first, daemon later

- Good, because it has the lowest barrier to entry.
- Bad, because approvals need the daemon or host prompts.

### Daemon from v0.1

- Good, because approvals are centralized.
- Bad, because it adds latency and setup cost to the basic path.

### Cloud service

- Good, because it is always available.
- Bad, because it contradicts the local-first principle ([ADR-0007](0007-no-telemetry.md)) and adds network dependency.

## More information

The local API over the daemon is specified in [ADR-0023](0023-json-rpc-local-api.md). The deployment models page describes local-first and team-server options.
