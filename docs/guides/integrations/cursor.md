<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.2 reviewed=2026-10-16 -->

# Cursor

> [!NOTE]
> **Guide: draft.**
> The adapter exists in a source build and has not been tested against a running Cursor.

**Enforcement level: L1 cooperative.** See
[enforcement levels](../../security/enforcement-levels.md).

## Configure the hooks

Cursor runs a hook per event rather than one hook for every tool, and reads back a
`permission` of `allow`, `deny`, or `ask`
(verified against <https://cursor.com/docs/hooks> on 2026-09-14). Put this in
`.cursor/hooks.json`:

```json
{
  "version": 1,
  "hooks": {
    "beforeShellExecution": [
      { "command": "shiin hook cursor", "failClosed": true }
    ],
    "beforeReadFile": [
      { "command": "shiin hook cursor", "failClosed": true }
    ],
    "beforeMCPExecution": [
      { "command": "shiin hook cursor", "failClosed": true }
    ]
  }
}
```

`failClosed: true` matters more here than anywhere else. Without it, Cursor **allows** the
action when a hook fails, which is the opposite of what you installed Shiin for.

## What it maps

| Cursor event | Action type | Notes |
|---|---|---|
| `beforeShellExecution` | `shell.exec` | Classified like any command, so a read it performs is recorded |
| `beforeReadFile` | `fs.read` | |
| `beforeMCPExecution` | `x-cursor.mcp.<tool>` | Opaque: a tool server describes its own behavior, and Shiin cannot verify it |
| Anything else | `x-cursor.<event>` | Opaque, so it falls through to a person or the default decision |

Shiin uses the first entry of `workspace_roots` as the workspace, falling back to `cwd`.

## Known gaps

- **Pending is denied by this adapter.** Live-host approval behavior has not been verified, so Shiin returns `deny` for both engine `deny` and `pending` decisions.
  The engine's pending reason remains in the local audit log.
  With the Unix daemon enabled, the response includes an approval ticket for separate operator resolution and retry; daemonless operation provides no ticket.
  See [Handling approvals](../handling-approvals.md).
- **Cloud agents skip the tool events**, so an action taken there never reaches Shiin.
- **MCP calls are opaque by design.** Precise mapping of tool calls arrives with the MCP
  proxy source implementation, which sits in the configured data path.

## Read next

The current [Cursor hook documentation](https://cursor.com/docs/hooks) describes `failClosed` and the response fields used by this adapter (verified 2026-10-15).
Automated protocol checks do not establish behavior in a running Cursor installation.

- [Claude Code](claude-code.md)
- [MCP proxy](mcp-proxy.md)
- [Limitations](../../concepts/limitations.md)
