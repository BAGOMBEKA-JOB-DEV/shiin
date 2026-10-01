<!-- shiin-doc: kind=policy status=accepted implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Privacy and telemetry

> [!NOTE]
> **Project policy: accepted.**
> This page is the only home of Shiin's privacy and telemetry rules ([ADR-0007](../adr/0007-no-telemetry.md)).

## No telemetry

Shiin collects no telemetry and has no phone-home behavior.
It needs no service and sends no data off the developer's machine.

## What stays local

All policy, audit logs, grants, tasks, and signing keys stay on the developer's machine.

## Network access

Every network feature is opt-in.
Shiin's engine performs no networking at all.

## Remote approvals

Remote approvals are planned for v0.6 and require an explicit opt-in ([ADR-0041](../adr/0041-remote-approver-transport.md)).

## Read next

- [No telemetry](../adr/0007-no-telemetry.md)
- [State and storage](../architecture/state-and-storage.md)