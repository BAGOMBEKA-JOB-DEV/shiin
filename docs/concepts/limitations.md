<!-- shiin-doc: kind=explanation status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# Limitations

> [!NOTE]
> **Design document: draft.**
> Describes the intended design. No implementation exists yet.

Shiin decides whether an action should happen before it runs.
These limits follow from where Shiin sits in the system.

## What Shiin cannot see

- **Code run by an allowed command.** Once a command is allowed, Shiin does not see what it does.
  A sandbox covers that gap.
- **Data leaving through an allowed action.** Shiin cannot inspect traffic content or see data leaving through an action it allowed.
  An egress proxy or DLP covers that gap.
- **Actions outside the interception point.** An L1 adapter cannot see actions that do not pass through the hook.
  The adapter's enforcement level states this honestly.

## What Shiin is not

Shiin is not a sandbox, a network firewall, a prompt-injection detector, or a DLP.
It complements those tools; it does not replace them.

## What Shiin cannot guarantee

- **No bypass.** A determined attacker with access to the operating system can bypass Shiin.
- **Perfect visibility.** Classifiers are conservative; anything unknown becomes opaque and needs approval.
- **Outcome correctness.** Shiin authorizes an action; it does not guarantee the action's real outcome was correct.

## Read next

- [How Shiin works](how-shiin-works.md)
- [Enforcement levels](../security/enforcement-levels.md)
- [Using Shiin with sandboxes](../guides/using-shiin-with-sandboxes.md)