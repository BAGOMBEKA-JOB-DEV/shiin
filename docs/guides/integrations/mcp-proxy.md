<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.5 reviewed=2026-10-16 -->

# MCP proxy

> [!WARNING]
> **Guide: draft.**
> An early source implementation wraps one downstream stdio server and evaluates tool calls through the daemon. MCP conformance, identity binding, and independent review remain open.

## Enforcement level

L2 Mediated.
Shiin sits in the data path between the client and each MCP server.

## Interception

The `shiin-mcp-proxy` binary launches one downstream server, forwards `tools/list`, and sends each `tools/call` through the local daemon before forwarding it. Start it with `--workspace ABSOLUTE_PATH -- SERVER ABSOLUTE_PATH [ARG ...]`, and configure `SHIIN_DAEMON=1` or an absolute `SHIIN_SOCKET`.

The proxy maps each tool name to an `mcp.tool.*` action type, records tool arguments as a digest, and marks their effect opaque. It returns a caller-visible tool error, including an approval ticket when present, for `deny`, `pending`, or a daemon failure. Only `allow` reaches the downstream server. Other MCP methods are not proxied yet.

The target is the [MCP specification revision 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28), checked on 2026-10-16. The current proxy does not use tool annotations to change its decision; annotation handling and conformance remain open.

## What it cannot see

- Traffic not routed through the proxy.
- Tool annotations do not affect decisions in this implementation.
- Prompts, resources, and other MCP methods are not proxied yet.

## Read next

- [Adapter contract](../../spec/adapter-contract.md)
- [Govern MCP tools](../../overview/use-cases.md#uc-08-govern-mcp-tools)
