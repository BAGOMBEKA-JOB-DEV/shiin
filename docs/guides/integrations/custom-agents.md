<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.5 reviewed=2026-10-01 -->

# Custom agents

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.5. Nothing on this page works yet.

## Enforcement level

L2 Mediated.
The agent asks Shiin for decisions through the local API.

## Interception

A custom agent built with `shiin-client` calls the local API over a Unix domain socket or Windows named pipe.
The API is JSON-RPC 2.0.

## What it cannot see

- A decision made when the daemon is unreachable, which is a fail-closed `deny`.

## Read next

- [Local API](../spec/local-api.md)
- [Agent-agnostic contract](../spec/README.md)