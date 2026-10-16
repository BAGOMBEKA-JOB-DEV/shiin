<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-16 -->

# Writing policies

> [!NOTE]
> **Guide: draft.**
> Everything below works with a source build, except where it says planned.
> [Policy model](../spec/policy-model.md) is the normative source.

A policy is one TOML file, `.shiin/policy.toml`. It says what agents may attempt in this
workspace, and nothing it does not mention is allowed.

## Shape of the file

```toml
version = 1                 # the format version; only 1 is understood

[capabilities.workspace]
id = "workspace"
allow = true
action_types = ["fs.*", "shell.exec"]

[defaults]
decision = "deny"           # what happens when no rule matches; "pending" is allowed, "allow" is not

[[rule]]                    # repeat this block for each rule
id = "edit-source"          # unique, and it appears in decisions and the audit log
effect = "allow"            # "allow", "deny", or "pending"
action_types = ["fs.write"] # which kinds of action this rule is about
resources = ["./src/**"]    # which paths
commands = ["cargo test*"]  # which commands
agents = ["claude-code"]    # which agents; omit for all of them
reason = "..."              # shown when this rule decides

[risk]                      # signals that escalate a decision
opaque_command = "pending"
bulk_delete = { decision = "pending", threshold = 20 }
```

An unknown key is refused rather than ignored, so a typo fails closed instead of quietly
widening what is allowed.

## How a rule matches

A rule matches when every matcher it declares matches. A matcher you leave out is not a
constraint, so a rule with only `action_types` applies to every resource.

- **Action types** come from the [registry](../spec/action-types.md), such as `fs.read` or
  `shell.exec`. A trailing `*` is a wildcard, so `fs.*` covers the whole namespace.
- **Resources** are globs against canonical, absolute paths. `*` matches within a path
  segment and `**` matches across segments. A pattern beginning `./` is relative to the
  workspace, so it can never match a path outside it. A pattern beginning `~/` is relative
  to your home directory.
- **Commands** are globs against the command as a whole, so `cargo test*` covers
  `cargo test --all`. They are not regular expressions.
- **Agents** are identifiers such as `claude-code`.
- **Sessions** and **tasks** are exact identifier lists; a task selector cannot match when no task is active.

Capabilities are separate from rules.
Every action/effect class needs an explicit capability grant, and a matching capability denial wins over a grant.
The starter policy includes grants for file operations and shell commands; an older policy without declarations now produces `deny.capability`.
Add only the classes the agent needs before validating and testing a migration.
Agent selectors remain asserted identifiers, not proof of identity.

A rule also matches an action whose *effects* are of the type it names, not only an action
whose own type matches. This is what makes one rule cover both ways of reading a file:

```toml
[[rule]]
id = "no-secrets"
effect = "deny"
action_types = ["fs.read"]
resources = ["**/.env"]
```

That rule denies the file-read tool, and it denies `cat .env` as well, because the
classifier records that the command performs a read of that path.
A command whose behavior cannot be read, such as `python cleanup.py`, carries no such
detail: it becomes an opaque effect, which the `opaque_command` signal turns into a
decision for a person.

## How rules combine

1. A matching `deny` wins over everything, so rule order in the file never matters.
2. Otherwise a matching `pending` holds the action for a person.
3. Otherwise a matching `allow` permits it.
4. Otherwise the default decision applies, which is a denial.

Risk signals and task constraints can make a result stricter, never more permissive.
[Evaluation](../spec/evaluation.md) states this precisely.

## Risk signals

A signal does not decide; it escalates. Configure each one with the decision it escalates
to, and a threshold where it counts something:

| Signal | Fires when |
|---|---|
| `opaque_command` | An action carries an effect Shiin cannot read, such as an interpreter or a shell substitution |
| `bulk_delete` | A single action would delete more files than the threshold, which defaults to 20 |
| `destructive_vcs` | A command rewrites or discards version control history, such as a force push |
| `privilege_escalation` | A command asks for higher privileges, such as `sudo` |
| `secret_path` | An effect touches a path known to hold secrets |
| `outside_workspace` | A write lands outside the workspace |

The full definitions are in [risk signals](../spec/risk-signals.md).

## Testing a change

Validate the file, then decide a request with it before trusting it to an agent:

```console
$ shiin policy validate
$ shiin explain /tmp/request.json
```

`explain` prints the stage that decided and the rules that matched, which is usually enough
to see why a rule did not apply.
Recorded policy cases run through `shiin policy test`; see [testing policies](testing-policies.md).

## Not yet available

| Feature | Arrives |
|---|---|
| Separately trusted user/workspace layers and attested identity | still open |
| Writing rules directly in Cedar | after the policy language is stable |

## Read next

- [Policy model](../spec/policy-model.md)
- [Policy syntax](../spec/policy-syntax.md)
- [Limitations](../concepts/limitations.md)
