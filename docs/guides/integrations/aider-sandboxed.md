<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.6 reviewed=2026-10-16 -->

# Aider in a sandbox

> [!NOTE]
> **Guide: draft.**
> Shiin's Unix command wrapper exists in source. Automatic routing from a running Aider host has not been verified.

## Enforcement level

L1 Cooperative through a shell wrapper.

## Setup

The source build provides an explicit command wrapper:

```console
$ shiin exec --session EXAMPLE-session -- /absolute/path/to/EXAMPLE-program EXAMPLE-argument
```

The wrapper evaluates `shell.exec` with conservative opaque effects and binds the exact executable and argument list.
Only allow starts the executable, after the audit record and task counters have been written.
When a daemon is configured, pending returns a ticket; retry the same arguments and session after approval.
Without the daemon, pending returns exit code 2 and does not execute the command.

This command does not automatically intercept a host's shell calls.
Verify the host routes every intended command through the wrapper before relying on it, and use a sandbox for the code an allowed command runs.

## What it cannot see

- Commands typed directly at the shell without the wrapper.
- File edits, network calls, and child processes that do not pass through the wrapper.

## Read next

- [Using Shiin with sandboxes](../using-shiin-with-sandboxes.md)
- [Shell wrapper](../../architecture/overview.md)
