<!-- shiin-doc: kind=spec status=draft implementation=partial milestone=v0.5 reviewed=2026-10-16 -->

# Local API

> [!NOTE]
> **Specification: draft.**
> The Unix daemon implements JSON-RPC evaluation, policy validation, and operator approval methods; `shiin-client` exposes evaluation and policy validation.
> Windows named pipes and attested agent identity remain unimplemented.
>
> Version: `shiin.local-api/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

The local API lets custom agents and integrations ask Shiin for decisions ([ADR-0023](../adr/0023-json-rpc-local-api.md), [ADR-0024](../adr/0024-tokio-confined-async-runtime.md)).

## Transport

**[API-001]** The local API MUST run over a Unix domain socket on Linux and macOS, and a Windows named pipe on Windows.

**[API-002]** The transport MUST be local-only and MUST NOT accept connections from remote hosts.

## Protocol

**[API-003]** The protocol MUST be JSON-RPC 2.0.

**[API-004]** The methods are:

| Method | Description |
|---|---|
| `shiin.evaluate` | Evaluate an action request and return a decision. |
| `shiin.evaluate.unattended` | Evaluate without an available approver; `pending` becomes `deny`. |
| `shiin.policy.validate` | Validate a policy and return diagnostics. |
| `shiin.approve.resolve` | Resolve an approval request. |
| `shiin.pending` | List pending requests; requires the operator credential. |
| `shiin.grants` | List grants; requires the operator credential. |
| `shiin.revoke` | Revoke a request or grant; requires the operator credential. |

## Authentication

**[API-005]** The caller's identity MUST be established at the `attested-local` assurance level or higher. The current service verifies that Unix peers share the daemon user's UID but does not yet bind that credential to the request's agent identity.

## Failure

**[API-006]** Any error, timeout, or unreachable daemon MUST produce a fail-closed `deny`. `shiin-client` returns an error; callers MUST stop the requested action unless the decision is `allow`.

## Example

<!-- validate: spec/schemas/local-api.v1.schema.json -->
```json
{
  "jsonrpc": "2.0",
  "id": "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b",
  "method": "shiin.evaluate",
  "params": {
    "schema": "shiin.action-request/v1-draft.1",
    "id": "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0c",
    "session": "sess-001",
    "agent": { "id": "agent-001", "assurance": "asserted" },
    "action_type": "fs.read",
    "effects": [
      { "kind": "path", "value": "/home/dev/project/src/login.rs", "confidence": "exact" }
    ],
    "task": null,
    "policy_snapshot": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "host": { "name": "custom-agent" }
  }
}
```

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.
- Implementation update (2026-10-16): Documented the existing Unix daemon methods and Rust client. Windows transport and identity binding remain open.
