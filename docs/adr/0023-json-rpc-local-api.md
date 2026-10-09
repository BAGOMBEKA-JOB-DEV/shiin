<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0023: JSON-RPC local API

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Local API](../spec/local-api.md), [ADR-0022](0022-local-first-daemonless-deployment.md), [ADR-0017](0017-json-wire-format-and-schemas.md), [ADR-0042](0042-language-bindings.md)

## Context and problem statement

The daemon (`shiind`, v0.3) needs an API for clients: the CLI, approver applications, and embedded libraries.
The API must be request-response over a local transport, language-neutral, and versionable.
It must not require HTTP or a network stack for the local-first path.

## Decision drivers

- Must be local-only (no network required).
- Must be language-neutral for future bindings.
- Must handle error cases, not just success.
- Must be versionable over time.

## Considered options

1. JSON-RPC 2.0 over a Unix domain socket (and named pipes on Windows).
2. A REST/HTTP API over localhost.
3. A custom binary protocol.

## Decision outcome

Chosen option: "JSON-RPC 2.0 over a Unix domain socket", because it is language-neutral, request-response, and does not require an HTTP server or port.

- The local API is JSON-RPC 2.0, as specified in the [local API specification](../spec/local-api.md).
- Transport is a Unix domain socket on Unix-like systems and a named pipe on Windows.
- All messages use the JSON wire format ([ADR-0017](0017-json-wire-format-and-schemas.md)).
- The `shiin-client` library implements the client side for Rust.

### Consequences

- Good, because JSON-RPC is simple, well-understood, and widely implemented.
- Good, because Unix domain sockets have no port to configure and no firewall rules.
- Bad, because Windows requires named pipes, which is a separate code path.
- Bad, because JSON-RPC over sockets is not directly browser-accessible (not needed for local-first).

### Confirmation

- The [local API specification](../spec/local-api.md) defines the methods, parameters, and error codes.
- CI tests the client library against the daemon.

## Pros and cons of the options

### JSON-RPC over local sockets

- Good, because it is language-neutral and local-only.
- Bad, because it is not HTTP, so standard HTTP tools cannot inspect it easily.

### REST/HTTP

- Good, because it is universally understood.
- Bad, because it requires a port, an HTTP stack, and TLS for any remote future.

### Custom binary protocol

- Good, because it is efficient.
- Bad, because it is not self-describing and requires code generation for bindings.

## More information

The [`shiin-ipc` crate](https://github.com/BAGOMBEKA-JOB-DEV/shiin) (planned: v0.3) implements the transport layer. The [deployment models](../architecture/deployment-models.md) page describes when the daemon is needed.
