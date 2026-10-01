<!-- shiin-doc: kind=explanation status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# State and storage

> [!NOTE]
> **Design document: draft.**
> Describes the intended design. No implementation exists yet.

Shiin stores its state on the developer's machine and collects no telemetry ([ADR-0007](../adr/0007-no-telemetry.md)).

## Locations

| State | Location | Milestone |
|---|---|---|
| Policy | `.shiin/policy.toml` in the workspace | v0.1 |
| Audit log | `.shiin/audit.jsonl` in the workspace | v0.1 |
| Grants | `.shiin/grants.jsonl` in Shiin's state directory | v0.3 |
| Signing keys | Shiin's state directory, with restricted permissions | v0.4 |
| Tasks | `.shiin/tasks/` in the workspace | v0.2 |

## Durability

A non-read action MUST NOT proceed until its audit event is durably recorded ([ADR-0030](../adr/0030-audit-durability.md)).

## Redaction

The audit log MUST NOT contain secret values ([AUD-006](../spec/audit-events.md#required-fields)).

## Read next

- [Audit events](../spec/audit-events.md)
- [Architecture overview](overview.md)