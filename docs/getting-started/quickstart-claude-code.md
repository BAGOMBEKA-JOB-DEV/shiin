<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Quickstart: Claude Code

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet.

## Steps

1. [Install](installation.md) Shiin.
2. Create a workspace policy:

   ```console
   $ shiin init
   ```

3. Configure Claude Code to use Shiin's hook.
4. Start Claude Code in your workspace.
5. When an agent proposes an action, Shiin evaluates it against policy.

## Example

An agent tries to read `.env`.
Shiin denies it with reason code `deny.policy`.

## Read next

- [Your first policy](first-policy.md)
- [Claude Code](../guides/integrations/claude-code.md)