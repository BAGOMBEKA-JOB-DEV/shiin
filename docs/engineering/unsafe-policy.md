<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# Unsafe code policy

> [!NOTE]
> **Project policy: draft.**
> Where `unsafe` Rust is allowed, and what every use of it must include.
> The workspace-wide `forbid` already exists in `Cargo.toml`; `shiin-platform` does not exist.

Unsafe Rust turns off compiler checks that Shiin relies on to guard actions for an untrusted [agent](../glossary.md#agent).
A memory-safety bug in Shiin could let a crafted input change a decision, so unsafe code is forbidden everywhere except one small, heavily reviewed crate.
This page follows [ADR-0037](../adr/0037-unsafe-code-policy.md).

## The rule

- `unsafe_code = "forbid"` is set in `[workspace.lints.rust]` in the root `Cargo.toml`.
  Every crate that inherits the workspace lints, including `xtask` today, cannot contain `unsafe` blocks, `unsafe fn`, or `unsafe impl`.
- `forbid` cannot be overridden by an `allow` or `expect` attribute inside the crate.
- The only exception is `shiin-platform`.

## `shiin-platform`

- `shiin-platform` is created only when a concrete feature needs an operating-system facility that no safe crate provides.
  Until then, it does not exist.
- The pull request that creates it links an ADR recording the need and the safe alternatives that were ruled out.
- It contains thin wrappers around operating-system calls and nothing else: no policy logic, no parsing of untrusted formats, and no business rules.
- Every wrapper exposes a safe API.
  The crate has no `pub unsafe fn`, so unsafety never leaks to callers.
- Wrappers validate their inputs before the unsafe call and convert every failure into a typed error.
- It declares its own lint table instead of inheriting the workspace one, as described in [Coding standards](coding-standards.md).
- It depends on no other Shiin crate, and `shiin-engine` and `shiin-schema` never depend on it; see [Workspace and crates](workspace-and-crates.md).

A likely first need is identifying the client process of a Windows named pipe for the [local API](../spec/local-api.md), which safe crates may not cover.
That is an expectation, not a decision.

## Prefer safe wrappers

Before any code reaches for `unsafe`, try these in order:

1. The Rust standard library.
2. Crates that already encapsulate the unsafe call behind a safe, well-tested API.
   On Unix-like systems that is usually [`rustix`](https://github.com/bytecodealliance/rustix); for sockets and pipes it can be the async runtime itself.
3. Only then, a wrapper in `shiin-platform`.

On Windows, [`windows-sys`](https://github.com/microsoft/windows-rs) is the preferred source of declarations instead of hand-written `extern` blocks.
Its functions are raw bindings, so every call to them is itself unsafe and belongs in `shiin-platform`.

Hand-written `extern` declarations are not used when a maintained binding crate provides the function.
`transmute`, `static mut`, and exported symbols such as `#[unsafe(no_mangle)]` are not used.

## Requirements for every unsafe block

1. **A `// SAFETY:` comment** directly above the block, explaining which invariants make it sound and where they are established.
   `clippy::undocumented_unsafe_blocks` enforces the presence of the comment; review enforces its accuracy.
2. **One unsafe operation per block**, so each comment justifies exactly one thing.
   `shiin-platform` enables `clippy::multiple_unsafe_ops_per_block` for this.
3. **A `# Safety` section** on any private `unsafe fn`, stating what callers must guarantee.
   Unsafe operations inside an `unsafe fn` still need their own `unsafe` block (`unsafe_op_in_unsafe_fn`).
4. **The smallest possible scope.**
   Safe code does the preparation and the checking, and the unsafe block contains only the call.
5. **Tests** that exercise the safe wrapper on every supported platform the wrapper targets, including error paths.
6. **Miri** runs the crate's tests on a nightly toolchain in CI.
   Miri cannot execute most foreign-function calls, so tests that need the real operating system are skipped under `cfg(miri)`;
   Miri then covers the pure-Rust handling of pointers and buffers around the calls, and platform tests cover the rest.
   See [Testing strategy](testing-strategy.md).
7. **Two-maintainer review.**
   A change that adds or modifies unsafe code needs approval from two maintainers.
   While the project has fewer than two maintainers, it needs review from an outside reviewer experienced with unsafe Rust, named in the pull request.
   Unsafe changes are always [security-sensitive changes](security-sensitive-changes.md).

## Unsafe code in dependencies

Forbidding unsafe code in Shiin's own crates does not remove it from the binaries: many dependencies contain some.
Unsafe usage is one of the admission criteria in [Dependencies and supply chain](dependencies-and-supply-chain.md), and reviewers weigh:

- how much unsafe code the crate contains, and whether it is necessary for the crate's purpose;
- whether its unsafe blocks are documented and tested, for example with Miri or fuzzing;
- the crate's history of soundness advisories in the [RustSec](https://rustsec.org/) database and how quickly they were fixed;
- whether a safe alternative of similar quality exists.

Tools that count unsafe usage inform this review but do not decide it.

## Soundness bugs

A soundness bug in `shiin-platform`, or one in a dependency that affects Shiin, is handled as a potential vulnerability
through [Vulnerability disclosure](../security/vulnerability-disclosure.md), not as an ordinary bug.

## Scope notes

- The fuzz targets in `fuzz/` form a separate workspace and do not inherit the workspace lints.
  They contain no unsafe code of their own.
- Code generated by macros from dependencies is covered by the dependency review above.
