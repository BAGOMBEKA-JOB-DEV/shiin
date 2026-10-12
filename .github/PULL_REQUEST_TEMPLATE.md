## Summary

<!-- What changes, and why. One or two sentences. -->

## Related

<!-- Issue, ADR, or RFC. Write "none" if there is none. -->

## Type of change

- [ ] Documentation or specification
- [ ] Decision record (ADR) or RFC
- [ ] Repository tooling (`xtask`, CI, configuration)
- [ ] Product code

## Checklist

- [ ] Commits are signed off (`git commit -s`), as required by the [DCO](../CONTRIBUTING.md#developer-certificate-of-origin).
- [ ] The change is scoped to one topic, with no unrelated reformatting.
- [ ] Documentation is updated, including page metadata and banners if a feature's status changed.
- [ ] Tests or specification test vectors are added or updated, where applicable.
- [ ] `CHANGELOG.md` has an entry under Unreleased, for anything user-visible.
- [ ] New dependencies are justified here and satisfy the [dependency policy](../docs/engineering/dependencies-and-supply-chain.md).
- [ ] The checks pass locally: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask docs-check`, `rumdl check .`, `typos`, `lychee`, `mdbook build docs`.

## Security

- [ ] This change touches a path listed in [security-sensitive changes](../docs/engineering/security-sensitive-changes.md).

If checked, describe the threat-model impact and the test that covers it:

<!-- Threat-model impact. -->

## AI assistance

- [ ] An AI assistant helped produce this change.

If checked, say which parts, and confirm that a human verified every factual claim about third-party products.
