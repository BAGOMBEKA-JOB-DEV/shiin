<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-12 -->

# Installation

> [!NOTE]
> **Guide: draft.**
> Building from source works today. There are no releases yet, so there is nothing to
> install from a package manager, and no signed binaries to verify.

## Build from source

Shiin needs [Rust](https://rustup.rs/). The repository pins its toolchain, so rustup
selects the right version on its own.

```console
$ git clone https://github.com/BAGOMBEKA-JOB-DEV/shiin.git
$ cd shiin
$ cargo install --path crates/shiin-cli
```

That puts `shiin` in `~/.cargo/bin`. Check it:

```console
$ shiin doctor
```

`doctor` reports the workspace it would act on, whether a policy is present and valid,
and the enforcement level you are getting. It is the fastest way to see that a change
to your setup took effect.

## What installing does not do

Installing changes nothing about your agents.
Shiin only sees an action if a host sends it, so until you configure a host's hook,
Shiin is inert. The [quickstart](quickstart-claude-code.md) connects Claude Code.

Uninstalling has the same shape in reverse: removing the binary leaves the host's hook
pointing at nothing. Remove the hook configuration as well, or the host will report a
failing hook on every tool call.

## Planned

Signed release binaries, installers, a software bill of materials, and the verification
steps in [verifying releases](../security/verifying-releases.md) arrive with the first
release. See the [roadmap](../project/roadmap.md).

## Read next

- [Quickstart with Claude Code](quickstart-claude-code.md)
- [Your first policy](first-policy.md)
