<!-- shiin-doc: kind=adr status=proposed implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0019: Path canonicalization

> [!NOTE]
> **Architecture decision: proposed.**
> Awaiting implementation confirmation.

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

- Paths are resolved to absolute form, with symlinks and `..` segments expanded.
- Both the file-read tool and `cat .env` produce an `fs.read` effect on the same canonical path.
- This is proposed and awaits confirmation by the v0.1 implementation.

### Consequences

- Good, because policy can match on canonical paths regardless of how the agent reached them.
- Good, because a single deny rule covers file tools, shell commands, and scripts that read the same file.
- Bad, because resolving symlinks requires file system access in the classifier, which adds latency and failure modes.

### Confirmation

- Use-case [UC-02](../overview/use-cases.md#uc-02-keep-secrets-out-of-reach) requires the classifier to map both the file-read tool and `cat .env` to the same canonical path.
- If symlink resolution is unsuitable, a superseding ADR records the alternative.

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
