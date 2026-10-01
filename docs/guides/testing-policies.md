<!-- shiin-doc: kind=how-to status=draft implementation=none milestone=v0.2 reviewed=2026-10-01 -->

# Testing policies

> [!WARNING]
> **Design intent: not implemented.**
> Planned for v0.2. Nothing on this page works yet.

Test your policy with `shiin policy test` against recorded cases.

## Test vectors

Test vectors encode the [golden use cases](../overview/use-cases.md) and are the conformance corpus.
Implementations run every vector and check the decision matches.

## Writing tests

Each test records one action request, the policy snapshot it is evaluated against, and the expected decision.

## CI

Policies are tested in CI before they are used in unattended environments.

## Read next

- [Test vectors](../spec/test-vectors/README.md)
- [Writing policies](writing-policies.md)