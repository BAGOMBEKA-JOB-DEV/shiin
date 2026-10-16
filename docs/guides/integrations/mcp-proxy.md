<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.5 reviewed=2026-10-16 -->

# MCP proxy

> [!NOTE]
> **Guide: draft.**
> A source implementation connects to one downstream stdio or Streamable HTTP server and evaluates tool calls through the daemon. External MCP conformance, identity binding, and independent review remain open.

## Enforcement level

L2 Mediated.
Shiin sits in the data path between the client and each MCP server.

## Interception

The `shiin-mcp-proxy` binary connects to one downstream server, negotiates MCP revision `2026-07-28` without legacy fallback, forwards `tools/list`, and sends each `tools/call` through the local daemon before forwarding it.
The upstream client always communicates with the proxy over stdio.
Configure `SHIIN_DAEMON=1` or an absolute `SHIIN_SOCKET`.

For a downstream subprocess, use `--workspace ABSOLUTE_PATH -- /ABSOLUTE/SERVER [ARG ...]`.
For a downstream HTTP server, use `--workspace ABSOLUTE_PATH --http https://mcp.example/mcp`.
These options are mutually exclusive.
The HTTP endpoint requires HTTPS except for literal loopback addresses; DNS names such as `localhost` do not enable the plaintext exception.
URLs containing credentials, query strings, or fragments are rejected.
Redirects, automatic HTTP retries, environment proxy discovery, and expired-session reconnection are disabled.
Connection setup is limited to ten seconds, with a five-second HTTP connection timeout and a sixty-second HTTP request timeout.
OAuth and authenticated remote endpoint configuration remain planned.

The proxy maps each tool name to an `mcp.tool.*` action type, records the full tool invocation as a digest, and marks its effect opaque. It returns a caller-visible tool error, including an approval ticket when present, for `deny`, `pending`, or a daemon failure. Only `allow` reaches the downstream server. Other MCP methods are not proxied yet.

The target is the [MCP specification revision 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28), checked on 2026-10-16. The current proxy does not use tool annotations to change its decision; annotation handling and conformance remain open.

The Rust integration test launches the actual proxy between an SDK client, a fixture server, and the Unix daemon service, with both downstream transports.
It covers discovery, allow/deny/pending, argument forwarding, daemon unavailability, and refusal to follow an HTTP redirect.
The test uses the [official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) (verified 2026-10-16); this coverage does not establish conformance against external reference servers.
The transport design follows the [MCP transport bindings](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports) (verified 2026-10-16).

## What it cannot see

- Traffic not routed through the proxy.
- Tool annotations do not affect decisions in this implementation.
- Prompts, resources, and other MCP methods are not proxied yet.

## Read next

- [Adapter contract](../../spec/adapter-contract.md)
- [Govern MCP tools](../../overview/use-cases.md#uc-08-govern-mcp-tools)
