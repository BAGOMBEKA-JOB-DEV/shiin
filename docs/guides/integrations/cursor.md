<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.2 reviewed=2026-10-12 -->

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

- **Whether `ask` is honored** for shell and tool events is not verified. A pending
  decision is reported as `ask` and the reason is also put in both messages, so that if
  Cursor treats it as an allow, at least the person sees why a human was wanted. Until
  this is confirmed, set risk signals you care about to `deny` rather than `pending` in
  policy.
- **Cloud agents skip the tool events**, so an action taken there never reaches Shiin.
- **MCP calls are opaque by design.** Precise mapping of tool calls arrives with the MCP
  proxy in v0.5, which sits in the data path instead of relying on a hook.

## Read next

- [Claude Code](claude-code.md)
- [MCP proxy](mcp-proxy.md)
- [Limitations](../../concepts/limitations.md)
