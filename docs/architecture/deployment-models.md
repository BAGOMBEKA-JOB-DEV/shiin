<!-- shiin-doc: kind=explanation status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# Deployment models

> [!NOTE]
> **Design document: draft.**
> Describes the intended design. No implementation exists yet.

Shiin is designed to run in several configurations.
The early design preserves the constraints those features will need.

## Local, daemonless

**v0.1.** A single user runs Shiin on their own machine.
The adapter runs as a process for each hook invocation.
No daemon, no service, no telemetry ([ADR-0022](../adr/0022-local-first-daemonless-deployment.md)).

## Daemon

**v0.3.** A per-user daemon (`shiind`) holds approval state and grants.
Adapters talk to it over the local API ([ADR-0023](../adr/0023-json-rpc-local-api.md)).

## Team

**After 1.0.** A team policy server provides shared policy, central evidence, and remote approvals.
It needs signed policy bundles and transport-independent APIs.

## Unattended

**v0.2.** In CI or other unattended environments, every `pending` becomes `deny`.
The policy is tested with `shiin policy test` before it is used.

## Read next

- [Architecture overview](overview.md)
- [State and storage](state-and-storage.md)