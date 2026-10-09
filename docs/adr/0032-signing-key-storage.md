<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0032: Signing key storage

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [ADR-0031](0031-dsse-ed25519-receipts.md), [Receipts](../spec/receipts.md), [Vision](../overview/vision.md#vision-statement), [ADR-0016](0016-built-in-protected-resources.md)

## Context and problem statement

Shiin signs checkpoints and receipts with Ed25519 keys.
Those keys must be stored in a way that is available to the signing process but not readable by agents.
The project needed a storage strategy that works on the developer's machine without requiring external infrastructure.

## Decision drivers

- Keys must not be accessible to agents or untrusted code.
- The storage must work on Linux, macOS, and Windows.
- Key rotation must be supported.

## Considered options

1. Store keys in the OS keyring (keychain on macOS, Credential Manager on Windows, Secret Service on Linux).
2. Store keys in a file protected by OS file permissions.
3. Generate keys on demand per session with no persistent storage.

## Decision outcome

Chosen option: "Store keys in the OS keyring", because it is the most portable way to protect secrets at rest without configuration.

- Ed25519 signing keys live in the OS keyring under the service name `shiin`.
- `shiin` reads the key for signing; agents cannot access the keyring through the adapter.
- The built-in protected resources ([ADR-0016](0016-built-in-protected-resources.md)) prevent policy from allowing agent access to the keyring.
- Key rotation is supported by `shiin key rotate`.
- If no key exists, `shiin init` generates one.

### Consequences

- Good, because the OS keyring is available on all Tier 1 platforms and protects keys at rest.
- Good, because no additional configuration is needed.
- Bad, because the keyring is per-user, so a team server scenario requires a different store (planned: after 1.0).

### Confirmation

- `shiin init` and `shiin key rotate` manage the key.
- The [receipts specification](../spec/receipts.md) references the key by its identifier, not its content.

## Pros and cons of the options

### OS keyring

- Good, because it is portable and protects keys at rest.
- Bad, because it is per-user and not suitable for a shared team server.

### File with OS permissions

- Good, because it is simple and works everywhere.
- Bad, because file permissions are coarser and can be bypassed by a process running as the same user.

### On-demand generation

- Good, because it needs no storage.
- Bad, because it cannot verify receipts from previous sessions.

## More information

The [cryptography page](../security/cryptography.md) lists the hash functions and signature algorithms. The threat model covers key-extraction attacks.
