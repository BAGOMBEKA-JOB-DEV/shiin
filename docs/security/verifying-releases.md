<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Verifying releases

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's release verification rules ([ADR-0039](../adr/0039-release-tooling.md)).

## What is signed

Every release binary is signed with Ed25519.
The signature covers the binary and its software bill of materials.

## How to verify

1. Download the binary and its signature.
2. Download the signing key.
3. Verify the signature with the key.

## Software bill of materials

Every release ships a software bill of materials listing every dependency.
It is signed alongside the binary.

## Key rotation

Signing keys are rotated on a schedule and on maintainer change.
Old keys are published and retained.

## Read next
- [Cryptography](cryptography.md)
- [Release engineering](../engineering/release-engineering.md)