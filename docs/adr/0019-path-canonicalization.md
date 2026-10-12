<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-10-12 -->

# ADR-0019: Path canonicalization

> [!NOTE]
> **Architecture decision: accepted.**
> The rules below are settled. Windows-specific details are deferred to v0.6.

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [ADR-0018](0018-conservative-action-classification.md), [Action request](../spec/action-request.md), [Use case UC-02](../overview/use-cases.md#uc-02-keep-secrets-out-of-reach)

## Context and problem statement

An agent might read a secret file through a file tool, or through a command like `cat .env`, or through a symlink, or through a relative path.
For policy to be effective, the classifier must map all of these to the same canonical path so that a single deny rule covers every form.
The project needed a canonicalization strategy that handles symlinks, parent-directory traversal, and environment-variable expansion.

## Decision drivers

- Policy must match on canonical paths, not on how the agent spelled the path.
- Shell commands and file tools must map to the same resource.
- The canonicalization must be deterministic and platform-consistent.

## Considered options

1. Canonicalize by resolving symlinks and `..`, then normalizing to an absolute path.
2. Match on the path string as the agent provides it, without canonicalization.
3. Resolve paths at the host level before handing them to the adapter.

## Decision outcome

Chosen option: "Canonicalize by resolving symlinks and `..`", because string matching allows bypasses through symlinks and traversal sequences.

Canonicalization runs in the adapter's classifier, before policy is consulted, and follows these rules in order:

1. **Make absolute.** A relative path resolves against the request's workspace root (AR-008). A path with no workspace context that is not absolute is unclassifiable, which makes its effect opaque.
2. **Expand `~` when policy is compiled,** not when a request is classified. The operator's home directory is a property of the machine, so a policy glob such as `~/.ssh/**` becomes an absolute pattern at load time. Requests never carry `~`.
3. **Resolve symlinks on the longest existing prefix.** Shiin resolves the deepest part of the path that exists, then appends the remaining segments lexically. This keeps `fs.create` classifiable for a file that does not exist yet.
4. **Resolve `.` and `..` lexically, after symlink resolution,** so that a symlink cannot be stepped over by a traversal sequence.
5. **Normalize Unicode to NFC,** so that two spellings of the same name cannot be treated as different paths.
6. **Preserve case, and compare case-insensitively only where the filesystem is case-insensitive.** Comparison is case-sensitive on Linux, and case-insensitive on macOS and Windows defaults. The Windows specifics (`\\?\` prefixes, 8.3 short names, junctions, and reparse points) are settled when Windows moves toward Tier 1 in v0.6.
7. **Fail closed.** If canonicalization errors, the effect is opaque and the request can never be allowed by a path rule.
8. **A workspace-relative pattern can never match outside the workspace** (AR-009), even if a symlink inside the workspace points out of it, because matching happens after resolution.

Both the file-read tool and `cat .env` therefore produce an `fs.read` effect on the same canonical path, which is what makes a single deny rule cover both.

### Consequences

- Good, because policy can match on canonical paths regardless of how the agent reached them.
- Good, because a single deny rule covers file tools, shell commands, and scripts that read the same file.
- Bad, because resolving symlinks requires file system access in the classifier, which adds latency and failure modes.

### Confirmation

- Use-case [UC-02](../overview/use-cases.md#uc-02-keep-secrets-out-of-reach) requires the classifier to map both the file-read tool and `cat .env` to the same canonical path.
- The `shiin-classify` crate owns rules 1 to 7, and property tests assert that canonicalization is idempotent and that no traversal or symlink sequence escapes a workspace-relative pattern ([testing strategy](../engineering/testing-strategy.md)).
- A regression test vector is added for every path-aliasing bypass that is reported.
- The time-of-check to time-of-use gap is a residual risk, not a defect: a path can be replaced between the decision and the action. It is recorded as out of scope in the [threat model](../security/threat-model.md) and is mitigated by a sandbox, not by Shiin.

## Pros and cons of the options

### Canonicalize with symlink resolution

- Good, because it prevents path-spoofing bypasses.
- Bad, because it requires file system access in the classifier.

### String match without canonicalization

- Good, because it is fast and has no I/O.
- Bad, because it allows bypasses through symlinks and traversal.

### Resolve at host level

- Good, because the host already has the canonical path.
- Bad, because not all hosts expose it, and the agent's path may differ from the canonical one.

## More information

The [conservative action classification ADR](0018-conservative-action-classification.md) describes how effects are derived from classified actions. The classification approach is a key part of [UC-02](../overview/use-cases.md#uc-02-keep-secrets-out-of-reach).
