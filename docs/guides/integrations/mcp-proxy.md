<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.5 reviewed=2026-10-01 -->

# MCP proxy

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.5. Nothing on this page works yet.

## Enforcement level

L2 Mediated.
Shiin sits in the data path between the client and each MCP server.

## Interception

The MCP proxy maps tool calls to [action requests](../glossary.md#action-request) and applies the same policy that hook adapters use.
MCP tool annotations can make a decision stricter but never more permissive.

## What it cannot see

- Traffic not routed through the proxy.

## Read next

- [Adapter contract](../spec/adapter-contract.md)
- [Govern MCP tools](../overview/use-cases.md#uc-08)