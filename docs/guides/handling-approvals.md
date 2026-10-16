<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.3 reviewed=2026-10-15 -->

# Handling approvals

> [!NOTE]
> **Guide: draft.**
> Local approval services and CLI administration are implemented in source for Unix.
> Host validation, Windows transport, and independent security review remain open.

## Start the local service

Build the workspace and run the daemon in a dedicated operator terminal:

```console
$ cargo build --workspace
$ shiind --operator
```

The daemon prints a fresh operator credential only after acquiring its exclusive state lock and binding its local socket.
Keep that credential in your terminal; never give it to an agent or put it in a command argument, environment variable, or workspace file.
It changes when the daemon restarts, while pending requests and grants remain in protected per-user state.

Enable the daemon explicitly for hook invocations with `SHIIN_DAEMON=1`.
The default endpoint is `~/.shiin/runtime/shiind.sock`; an absolute `SHIIN_SOCKET` override selects another protected directory.
An explicitly configured daemon that is unreachable causes denial, with no fallback to daemonless evaluation.

## Inspect and resolve actions

From your own terminal:

```console
$ shiin pending
$ shiin approve EXAMPLE-TICKET allow_once 300
$ shiin approve EXAMPLE-TICKET deny 300
$ shiin approve EXAMPLE-TICKET allow_always 3600 scope.json
$ shiin grants
$ shiin revoke EXAMPLE-GRANT-ID
```

Replace the example identifiers with the ticket or grant shown by Shiin.
Each command reads the operator credential with terminal echo disabled.
Approval displays the normalized action and requires a typed confirmation before resolution.
A scoped grant requires an explicit JSON scope file with `agent`, exact `action_types`, `resource`, and `workspace` fields.
The resource glob applies to every path effect; the explicitly selected pattern `*` also covers non-path effects.
Grants remain bound to the session, active task, and policy snapshot, and expire after at most 24 hours or the task deadline, whichever comes first.

Agent-facing requests cannot resolve, list, or revoke approvals without the separate operator credential.
The built-in policy denies intercepted `shiin approve`, `shiin revoke`, and `shiind` commands.
This is cooperative enforcement: code already allowed to run outside interception is not isolated from the operating-system account.

## As an approver

When an action is `pending`, an [approval request](../glossary.md#approval-request) is created.
You can:

- **Allow once.** The action proceeds; the grant is consumed.
- **Allow for a scope and time.** The action and similar ones proceed until the grant expires.
- **Deny.** The action is refused.

## As a developer

With the daemon enabled, pending actions are denied immediately with a ticket.
Approve from your separate terminal, then retry the action.
The adapter recomputes its normalized binding rather than trusting an agent's ticket or description.

## Deny-with-ticket

For a host that cannot wait, the adapter denies the action now and returns an approval ticket.
If the agent retries after an approver approves, the retry consumes a single-use grant.

## Expiry

An approval request that is never resolved expires as `deny`.

## Read next

- [Approvals](../concepts/approvals.md)
- [Approval protocol](../spec/approval-protocol.md)
