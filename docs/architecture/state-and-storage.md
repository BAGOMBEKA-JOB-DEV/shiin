<!-- shiin-doc: kind=explanation status=draft implementation=partial milestone=m0 reviewed=2026-10-16 -->

# State and storage

> [!NOTE]
> **Design document: draft.**
> The local authorization core, approval service, and signed evidence paths exist in source; platform support and milestone validation remain incomplete.

Shiin stores its state on the developer's machine and collects no telemetry ([ADR-0007](../adr/0007-no-telemetry.md)).

## Locations

| State | Location | Milestone |
|---|---|---|
| Policy | `.shiin/policy.toml` in the workspace | v0.1 |
| Audit log | `.shiin/audit.jsonl` in the workspace | v0.1 |
| Approval requests and grants | `~/.shiin/runtime/approvals.json`, beside the Unix daemon socket | v0.3, source implementation |
| Signing keys | OS keyring, following [ADR-0032](../adr/0032-signing-key-storage.md) | v0.4, partial source implementation |
| Tasks | `.shiin/tasks/` in the workspace | v0.2 |
| Receipts and encrypted replay | `.shiin/evidence/<request-id>.receipt.json` and `<request-id>.replay.json` | v0.4, when daemon evidence is enabled |
| Signed checkpoint and public keys | `.shiin/evidence/checkpoint.json` and `<key-id>.public.json` | v0.4, when daemon evidence is enabled |

An absolute `SHIIN_SOCKET` changes the daemon directory and places `approvals.json` beside that socket.
The directory is private to the current Unix user; operator credentials are held in memory and change on restart.

## Durability

A non-read action does not proceed until its audit event is durably recorded ([ADR-0030](../adr/0030-audit-durability.md)).

## Redaction

The audit log never contains secret values ([AUD-006](../spec/audit-events.md#required-fields)).

## Read next

- [Audit events](../spec/audit-events.md)
- [Architecture overview](overview.md)
