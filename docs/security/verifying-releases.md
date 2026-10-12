<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.1 reviewed=2026-10-12 -->

# Verifying releases

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.1. Nothing on this page works yet, because nothing has been released.
> The release workflow exists and is described below, but it has never run, so treat every
> command here as the intended procedure rather than a tested one.

A release is built by a workflow in this repository, from a tag, on GitHub's runners.
Verifying one answers two questions: did I download what was published, and was it built
from this repository's source.

## 1. Check the archive against its checksum

Every archive is published with a `.sha256` file beside it.

```console
$ sha256sum --check shiin-cli-x86_64-unknown-linux-gnu.tar.xz.sha256
```

A checksum only proves the download was not corrupted. It proves nothing about origin,
because whoever could replace the archive could replace the checksum.

## 2. Verify the build provenance

Each archive carries a signed attestation naming the repository, the workflow, and the
commit it was built from. That is what ties a binary to this source.

```console
$ gh attestation verify shiin-cli-x86_64-unknown-linux-gnu.tar.xz \
    --repo BAGOMBEKA-JOB-DEV/shiin
```

Expect the output to name this repository and the release workflow. If verification fails,
do not run the binary.

## 3. Read the bill of materials

Each release has a CycloneDX bill of materials, `shiin-<version>.cdx.json`, listing every
crate the binary was built from and its version.

```console
$ gh release download <version> --pattern '*.cdx.json'
```

Use it to check whether a published advisory affects a release you already have.

## What this does not tell you

- **That the source is correct.** Provenance ties a binary to a commit. Reviewing the
  commit is a separate thing.
- **That the build is reproducible.** Reproducible builds are a goal, not a promise; see
  [release engineering](../engineering/release-engineering.md).
- **That Shiin is enforcing anything.** A verified binary that no host calls does nothing.
  Run `shiin doctor` and read [enforcement levels](enforcement-levels.md).

## Planned

`cargo auditable` metadata, which would let `cargo audit bin` read the dependency list
straight out of the binary, needs support in the release tooling and is not wired up yet.

## Read next

- [Release engineering](../engineering/release-engineering.md)
- [Hardening](hardening.md)
