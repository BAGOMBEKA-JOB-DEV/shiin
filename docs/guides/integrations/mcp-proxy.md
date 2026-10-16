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
For a provisioned access token, append `--bearer-env SHIIN_EXAMPLE_MCP_TOKEN` and provide that variable through your operator-controlled credential setup.
Only the variable name goes in the command; keep token values out of arguments, workspace files, and agent input.
Bearer mode requires HTTPS even on loopback, rejects missing/invalid credentials before connecting, and marks the Authorization header sensitive so debug formatting does not reveal its value.
An optional `--ca-file ABSOLUTE_PATH` adds one operator-selected PEM trust root for a private HTTPS server; hostname and certificate verification remain enabled.
CA files are bounded to 64 KiB, and malformed or unavailable roots fail setup.
The provisioned-token path follows [Bearer header usage](https://www.rfc-editor.org/rfc/rfc6750#section-2.1) and the [MCP access-token requirements](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization#access-token-usage) (verified 2026-10-16).
OAuth discovery, browser consent, token acquisition, and refresh remain planned; a provisioned token does not establish complete MCP authorization conformance.

The proxy maps each tool name to an `mcp.tool.*` action type, records the full tool invocation as a digest, and marks its effect opaque. It returns a caller-visible tool error, including an approval ticket when present, for `deny`, `pending`, or a daemon failure. Only `allow` reaches the downstream server. Other MCP methods are not proxied yet.

The target is the [MCP specification revision 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28), checked on 2026-10-16. The current proxy does not use tool annotations to change its decision; annotation handling and conformance remain open.

The Rust integration test launches the actual proxy between an SDK client, a fixture server, and the Unix daemon service, with both downstream transports.
It covers discovery, allow/deny/pending, argument forwarding, daemon unavailability, refusal to follow an HTTP redirect, and an authenticated HTTPS fixture with an explicit generated test trust root.
Process tests also check that credential setup failures do not reveal token values.
The test uses the [official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) (verified 2026-10-16); this coverage does not establish conformance against external reference servers.
The transport design follows the [MCP transport bindings](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports) (verified 2026-10-16).

## What it cannot see

- Traffic not routed through the proxy.
- Tool annotations do not affect decisions in this implementation.
- Prompts, resources, and other MCP methods are not proxied yet.

## Read next

- [Adapter contract](../../spec/adapter-contract.md)
- [Govern MCP tools](../../overview/use-cases.md#uc-08-govern-mcp-tools)
