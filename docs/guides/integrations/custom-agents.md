<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.5 reviewed=2026-10-16 -->

# Custom agents

> [!NOTE]
> **Guide: draft.**
> The Unix local API and Rust client exist in source. Windows transport, attested agent identity, and independent validation remain open.

## Enforcement level

L2 Mediated.
The agent asks Shiin for decisions through the local API.

## Interception

A custom agent built with `shiin-client` calls the local API over a Unix domain socket.
The API uses JSON-RPC 2.0. The client returns an error for an unavailable daemon or malformed response; the caller must stop the action unless Shiin returns `allow`.

```rust
use shiin_client::Client;
use shiin_schema::DecisionValue;

let client = Client::from_environment()?;
let decision = client.evaluate(&request).await?;
if decision.decision != DecisionValue::Allow {
    return Err("Shiin did not allow the action".into());
}
```

## What it cannot see

- Agent identity assurance: the daemon authenticates the local Unix user but does not yet bind that identity to the request's agent id.
- Windows named-pipe transport.

## Read next

- [Local API](../../spec/local-api.md)
- [Agent-agnostic contract](../../spec/README.md)
