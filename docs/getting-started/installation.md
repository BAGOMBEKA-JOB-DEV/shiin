<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-18 -->

# Installation

> [!NOTE]
> **Guide: draft.**
> Source builds work today. The first alpha candidate is `v0.1.0-alpha.1`, with Linux and macOS binary distributions.
> A candidate becomes available for installation only after its GitHub release is published.
> Windows daemon transport and path qualification remain open; no Windows archive is included in this first alpha.

## Published alpha archives

Use the [releases page](https://github.com/BAGOMBEKA-JOB-DEV/shiin/releases) to select a published version.
The CLI archive is named `shiin-cli-<target>.tar.xz`; the optional daemon archive is `shiind-<target>.tar.xz`.
The targets are `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-apple-darwin`, and `x86_64-apple-darwin`.
Download the matching archive and its `.sha256` file, then follow [release verification](../security/verifying-releases.md) before running either executable.
Extract the verified archive and put its executable in a directory on your `PATH`.

The generated `shiin-cli-installer.sh` and `shiind-installer.sh` are alternatives available on the same release.
Review a downloaded installer before running it; installation does not start a service or change a host's hooks.
The daemon runs in the foreground when explicitly started with `shiind --operator` in a dedicated operator terminal.
Keep the displayed approval credential out of agent sessions.

Check the installed version with `shiin --version` and, if installed, `shiind --version`.
Linux signing/evidence initialization needs an available, unlocked Secret Service; unavailable keys fail without a file fallback.

## Build from source

Shiin needs [Rust](https://rustup.rs/). The repository pins its toolchain, so rustup
selects the right version on its own.

```console
$ git clone https://github.com/BAGOMBEKA-JOB-DEV/shiin.git
$ cd shiin
$ cargo install --locked --path crates/shiin-cli
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

To build the optional Unix daemon, use `cargo install --locked --path crates/shiind`.
The MCP proxy and client library remain source integrations, with their limitations described in the [roadmap](../project/roadmap.md).

## Read next

- [Quickstart with Claude Code](quickstart-claude-code.md)
- [Your first policy](first-policy.md)
