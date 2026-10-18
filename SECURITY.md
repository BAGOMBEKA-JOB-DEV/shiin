# Security policy

Shiin is experimental and has no stable release or security-audit claim.
The latest published alpha receives security fixes; older alpha versions receive no backports.
Reports about the implementation and design are welcome.

## Reporting a vulnerability

Report privately through GitHub Private Vulnerability Reporting, at
<https://github.com/BAGOMBEKA-JOB-DEV/shiin/security/advisories/new>.

Please do not open a public issue, pull request, or discussion for a suspected vulnerability.

## What counts as a vulnerability

Shiin's purpose is to decide whether an AI agent's action is authorized, so anything that defeats a decision is in scope:

- Making the engine return `allow` where policy, a task constraint, or a risk signal should have produced `deny` or `pending`.
- Evading classification so that an action is not seen for what it is, such as path aliasing or command obfuscation.
- Changing Shiin's own state, policy, grants, audit log, or a host's hook configuration through an action Shiin mediates.
- Approving an action without an authorized approver, or replaying an approval.
- Forging or silently altering audit events or receipts.
- Causing a failure that results in an action being allowed instead of denied.

The [threat model](docs/security/threat-model.md) states what is deliberately out of scope, including the effects of code that was legitimately allowed to run, actions that never pass through an adapter, and an attacker who already controls the operating system account.
If you are unsure which side of the line a finding falls on, report it and say why.

## What to include

- What you were trying to defeat, and what happened instead.
- The exact action request or command, the policy in effect with secrets removed, and the decision you observed or expect.
- The affected specification, page, or requirement identifier.
- Your assessment of impact.

## Our commitments

- Acknowledge your report privately within 3 business days.
- Give an initial assessment within 10 business days.
- Keep you informed while we work, and agree a disclosure date with you, normally within 90 days.
- Credit you in the advisory unless you prefer otherwise.
- Never take legal action against good-faith research that follows this policy and avoids privacy violations, data destruction, and service disruption.

Severity is scored with CVSS as published through GitHub security advisories.
The full process, including the maintainer runbook, is in [vulnerability disclosure](docs/security/vulnerability-disclosure.md).
