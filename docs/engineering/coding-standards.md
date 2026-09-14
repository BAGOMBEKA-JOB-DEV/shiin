<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# Coding standards

> [!NOTE]
> **Project policy: draft.**
> Rules for how Rust code in this repository is formatted, linted, named, documented, and logged, and how commits are written.
> The formatter settings and the workspace lint table already exist; per-crate additions are planned.

These standards apply to every crate in the workspace, including `xtask`.
Most of them are enforced by tools, so a pull request that passes CI already follows them.
The rest are checked in review.

## Formatting

- Code is formatted with `rustfmt` using the repository's `rustfmt.toml`:

  ```toml
  style_edition = "2024"
  ```

- CI runs `cargo fmt --all --check`, and a pull request with unformatted code fails.
- `#[rustfmt::skip]` is used only where formatting destroys meaning, such as an aligned table of constants, with a comment saying why.
- Editors pick up whitespace rules from `.editorconfig`:

  ```text
  root = true

  [*]
  charset = utf-8
  end_of_line = lf
  insert_final_newline = true
  trim_trailing_whitespace = true
  indent_style = space
  indent_size = 4

  [*.{md,toml,yml,yaml,json}]
  indent_size = 2
  ```

## Lints

### The workspace lint table

Every crate inherits this table from the root `Cargo.toml` with `[lints] workspace = true`.
It is quoted here exactly as it appears in the file:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
unreachable_pub = "warn"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"
todo = "deny"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "deny"
allow_attributes_without_reason = "deny"
undocumented_unsafe_blocks = "deny"
```

### Why each forbid and deny exists

A panic or stray output in a hook process is not just a bug.
[Hosts](../glossary.md#host) can treat a crashed hook as permission to proceed, and some parse a hook's output as its answer,
so these lints protect [fail-closed](../glossary.md#fail-closed) behavior as much as code quality.

| Lint | Level | Why |
|---|---|---|
| `unsafe_code` | forbid | Unsafe code is not allowed outside `shiin-platform`. `forbid`, unlike `deny`, cannot be overridden by an `allow` or `expect` inside the crate. See [Unsafe code policy](unsafe-policy.md). |
| `clippy::unwrap_used` | deny | `unwrap()` turns an unexpected `None` or `Err` into a panic. Errors are propagated with `?` and handled as described in [Error handling](error-handling.md). |
| `clippy::panic` | deny | An explicit `panic!` in a hook or the engine can crash the process that was supposed to answer `deny`. |
| `clippy::todo` | deny | Unfinished code does not merge. A `todo!` reached at run time panics. |
| `clippy::dbg_macro` | deny | `dbg!` writes to standard error, bypasses logging configuration, and can print sensitive values. |
| `clippy::print_stdout` | deny | In a hook, standard output carries the response the host parses, so a stray line can corrupt the decision. Libraries never print. |
| `clippy::print_stderr` | deny | Standard error can reach the host and the agent, so stray output can leak internal detail. Diagnostics go through `tracing` or a binary's dedicated output module. |
| `clippy::allow_attributes_without_reason` | deny | Every lint suppression states why it is needed, so reviewers can judge it and remove it when the reason is gone. |
| `clippy::undocumented_unsafe_blocks` | deny | Every `unsafe` block carries a `// SAFETY:` comment. It matters only in `shiin-platform`, but it is set everywhere so the rule cannot be forgotten. |

The warn-level entries matter too, because CI denies all warnings:

| Lint | Level | Why |
|---|---|---|
| `missing_docs` | warn | Public items are documented; see [Documentation comments](#documentation-comments). |
| `unreachable_pub` | warn | Items marked `pub` that are not reachable from the crate's public API become `pub(crate)`, so the public surface that SemVer checks see is the real one. |
| `clippy::pedantic` | warn, priority -1 | Enables the pedantic group. The lower priority lets individual entries in the same table override lints from the group. |
| `clippy::expect_used` | warn | `expect` is allowed only where an invariant makes failure impossible, and the suppression must say which invariant. |

`clippy::panic` and `clippy::todo` do not cover `unimplemented!`, indexing, or arithmetic overflow.
Reviewers treat `unimplemented!` like `todo!`, and the engine crates add the lints below for indexing and arithmetic.

### Warnings in CI

CI runs Clippy with every warning denied:

```console
$ cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Locally, warn-level lints stay warnings, so work in progress still compiles.
Nothing with a warning merges.
The documentation workflow runs the same command without `--all-features` today, because `xtask` defines no features; see [Continuous integration](ci.md).

### Planned additions for the engine, classification, and policy crates

`shiin-engine`, `shiin-classify`, and `shiin-policy` handle input that an untrusted [agent](../glossary.md#agent) controls, such as paths, command lines, and counters.
When they are created, they add two lints:

| Lint | Level | Why |
|---|---|---|
| `clippy::indexing_slicing` | deny | Indexing with `[]` panics when out of bounds. These crates use `get` and handle `None`. |
| `clippy::arithmetic_side_effects` | warn | Integer overflow panics in debug builds and wraps silently in release builds, and a wrapped counter could let an agent exceed a task limit. These crates use checked or saturating arithmetic. |

Cargo does not let a crate combine `[lints] workspace = true` with its own lint entries,
so these crates declare the additions as attributes at the top of `lib.rs`:

```rust
// crates/shiin-engine/src/lib.rs (planned)
#![deny(clippy::indexing_slicing)]
#![warn(clippy::arithmetic_side_effects)]
```

### The `shiin-platform` exception

`shiin-platform`, if it is created, is the only crate that does not inherit the workspace table.
A crate that inherits `unsafe_code = "forbid"` cannot allow unsafe code anywhere, so `shiin-platform` declares its own `[lints]` table instead.
That table copies every workspace entry, changes `unsafe_code` to `deny` so individual modules can opt in with `#[expect]`,
and adds stricter lints for unsafe code:

```toml
# crates/shiin-platform/Cargo.toml (planned; the other workspace entries are copied unchanged)
[lints.rust]
unsafe_code = "deny"
unsafe_op_in_unsafe_fn = "deny"

[lints.clippy]
undocumented_unsafe_blocks = "deny"
multiple_unsafe_ops_per_block = "deny"
```

A planned `xtask` check compares this table with the workspace table so the copy cannot drift.

### Suppressing a lint

- Suppress a lint with `#[expect(lint, reason = "...")]`, not `#[allow]`.
  `expect` produces a warning when the lint stops firing, so stale suppressions are found and removed.
- Put the attribute on the smallest item that needs it.
- The reason says why the lint is wrong in this case, not what the lint does.
- `#[allow(lint, reason = "...")]` is acceptable only where the lint fires on some targets or feature combinations but not others,
  because `expect` would then warn on the configurations where the lint does not fire.
- Today, the workspace table enforces the reason, and review enforces the preference for `expect`.

`xtask` shows the pattern for a binary that legitimately writes to the terminal:

```rust
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "docs-check is a terminal command that reports its findings"
)]
```

## Naming and API design

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), in particular their naming, conversion (`as_`, `to_`, `into_`), and getter conventions.
- Use [glossary](../glossary.md) terms for type and function names: `ActionRequest`, `Decision`, `PolicySnapshot`, `ApprovalRequest`, and `Grant`, not synonyms.
- Decision values are `Allow`, `Deny`, and `Pending` in Rust, matching `allow`, `deny`, and `pending` on the wire.
- Engine and policy types never contain host tool names; host names appear only in adapter modules.
- Public enums that may gain variants are `#[non_exhaustive]`.
- Identifiers such as agent IDs and request IDs are newtypes, not bare `String` or `u64` values.
- Functions that return a value the caller should not ignore, such as a `Decision`, are `#[must_use]`.

## Documentation comments

- Every public item has a doc comment. `missing_docs` enforces this.
- Public functions, types, and traits include at least one example in their doc comment.
  Examples are doctests, so they compile and run in CI.
- Functions that return `Result` have an `# Errors` section, and functions that can panic have a `# Panics` section.
  The pedantic Clippy group checks both.
- Every crate has crate-level documentation (`//!`) stating its responsibility and, before 1.0, whether its API is internal.
- Doc comments describe behavior and contracts.
  When a normative specification defines the behavior, the comment links to it instead of restating it.

## Logging

- Code logs with [`tracing`](https://github.com/tokio-rs/tracing) events and spans.
- Libraries emit events but never install a subscriber, and they never print.
  Binaries configure where logs go.
- The hook path never writes logs to standard output, which carries the host protocol.
  Where log files live is described in [State and storage](../architecture/state-and-storage.md).
- `shiin-engine` does not log.
  It returns a [trace](../glossary.md#trace) as part of its result, which keeps it pure and makes explanations reproducible.
- Levels: `error` for failures that lead to a fail-closed `deny`, `warn` for degraded operation, `info` for lifecycle events, and `debug` or `trace` for development detail.

### No secrets in logs

- Never log raw tool input, file contents, command output, environment variables, tokens, approval secrets, or key material.
- Log identifiers and digests instead, such as a request ID or a payload hash.
- Types that hold secrets implement `Debug` by hand so that they print a placeholder, never their contents.
- The [audit log](../glossary.md#audit-log) is not a log in this sense; its redaction rules are in [Audit events](../spec/audit-events.md).
- See also [Privacy and telemetry](../project/privacy-and-telemetry.md).

## Commit messages

Commits follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/),
because release tooling generates changelogs from them (see [Release engineering](release-engineering.md)).

- The subject is `type(scope): summary`, written in the imperative and in lowercase, with no trailing period.
- Types are `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci`, and `chore`.
- The scope is the crate name without the `shiin-` prefix (`engine`, `classify`), `xtask`, or a documentation area (`spec`, `security`).
- A breaking change adds `!` after the scope and a `BREAKING CHANGE:` footer explaining the migration.
- Every commit carries a Developer Certificate of Origin sign-off, which `git commit -s` adds.

```console
$ git commit -s -m "fix(classify): treat unrecognized shell builtins as opaque"
```
