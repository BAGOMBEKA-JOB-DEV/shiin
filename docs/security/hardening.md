<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Hardening

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's hardening guidance.

## Run agents as a less-privileged user

Where possible, run agents as a less-privileged user.
Shiin is not a replacement for operating-system permissions.

## Keep host controls enabled

Shiin's L1 adapters use host hooks, which can be disabled.
Keep the host's own controls enabled alongside Shiin.

## Use a sandbox

Shiin decides whether an action should happen.
A sandbox limits the damage if an allowed action, or the code it runs, misbehaves.

## Keep Shiin updated

Shiin is updated through ordinary pull requests and releases.
Signed release binaries with a software bill of materials are produced for every release.

## Verify releases

See [Verifying releases](verifying-releases.md).

## Read next
- [Verifying releases](verifying-releases.md)
- [Using Shiin with sandboxes](../guides/using-shiin-with-sandboxes.md)