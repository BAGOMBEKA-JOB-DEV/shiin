<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Action requests

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

An [action request](../spec/action-request.md) is the normalized, host-independent description of one proposed [action](../glossary.md#action).

## What it contains

An action request records:

- Which [agent](../glossary.md#agent) is asking, and its [assurance level](../glossary.md#assurance-level).
- The [action type](../spec/action-types.md), such as `fs.write`.
- The [effects](../glossary.md#effect): what resources the action touches, with a confidence.
- The active [task](../glossary.md#task-declaration), if any.
- The [policy snapshot](../glossary.md#policy-snapshot) digest.

## How an adapter builds it

An [adapter](../glossary.md#adapter) turns a host payload into an action request.
It classifies the action and its effects.
Anything it cannot classify becomes an [opaque effect](../glossary.md#opaque-effect).

## Example

An agent edits `src/login.rs` through a file-write tool.
The adapter builds a request with action type `fs.write` and one effect: an exact write to `/home/dev/project/src/login.rs`.

## Read next

- [Action request](../spec/action-request.md)
- [Action types](../spec/action-types.md)