<!-- shiin-doc: kind=how-to status=draft implementation=partial milestone=v0.1 reviewed=2026-10-18 -->

# Verifying releases

> [!NOTE]
> **Guide: draft.**
> The first alpha publishes archive checksums, signed build provenance, dependency metadata, and SBOMs.
> Apply these steps to the assets on the [versioned release page](https://github.com/BAGOMBEKA-JOB-DEV/shiin/releases/tag/v0.1.0-alpha.1).

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

Each release includes `shiin-cli.cdx.xml` and `shiind.cdx.xml`, listing package dependencies and versions in CycloneDX format.

```console
$ gh release download <version> --pattern '*.cdx.xml'
```

Use it to check whether a published advisory affects a release you already have.

## What this does not tell you

- **That the source is correct.** Provenance ties a binary to a commit. Reviewing the
  commit is a separate thing.
- **That the build is reproducible.** Reproducible builds are a goal, not a promise; see
  [release engineering](../engineering/release-engineering.md).
- **That Shiin is enforcing anything.** A verified binary that no host calls does nothing.
  Run `shiin doctor` and read [enforcement levels](enforcement-levels.md).

## Embedded dependency metadata

The release configuration enables `cargo auditable` metadata in binaries.
The [dist option](https://axodotdev.github.io/cargo-dist/book/reference/config.html#cargo-auditable) selects the auditable build command (verified 2026-10-18).
The archive provenance and checksums remain the origin and integrity checks; embedded dependency metadata does not replace them.

## Read next

- [Release engineering](../engineering/release-engineering.md)
- [Hardening](hardening.md)
