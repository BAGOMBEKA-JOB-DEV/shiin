<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

# Security-sensitive changes

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's security-sensitive change rules.

## What is security-sensitive

A change is security-sensitive if it affects:

- The decision pipeline, including any evaluation stage.
- Policy compilation or evaluation.
- The audit log, receipts, or key management.
- Unsafe code in `shiin-platform` or dependencies.
- Adapter enforcement or fail-closed behavior.

## Review

Security-sensitive changes need:

- A link to the relevant ADR or RFC.
- Review from a maintainer experienced in the area.
- For unsafe code, two-maintainer review or an outside reviewer.

## Disclosure

A change that might be a way to bypass Shiin is handled through [Vulnerability disclosure](vulnerability-disclosure.md), not as an ordinary bug.

## Read next
- [Vulnerability disclosure](vulnerability-disclosure.md)
- [Unsafe code policy](unsafe-policy.md)