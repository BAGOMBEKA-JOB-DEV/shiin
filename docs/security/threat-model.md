<!-- shiin-doc: kind=spec status=draft implementation=partial milestone=m0 reviewed=2026-10-16 -->

# Threat model

> [!NOTE]
> **Specification: draft.**
> Implemented in part in the source build. Remaining requirements and external validation are not complete.

This page defines the threats Shiin is designed to resist.
Threats are referenced throughout the documentation as `**[ST-<category>-<nn>]**`.

## STRIDE categories

| Category | Meaning |
|---|---|
| S | Spoofing identity |
| T | Tampering with data or logs |
| R | Repudiation (denying an action) |
| I | Information disclosure |
| D | Denial of service |
| E | Elevation of privilege |

## Threats

### Spoofing

**[ST-S-01]** An agent presents a false identity.
Shiin treats the agent as untrusted and never allows an action because the agent says it is safe.

**[ST-S-02]** An approver is impersonated.
Approver identity is established through an approval channel that the agent does not control.

### Tampering

**[ST-T-01]** An agent modifies the audit log.
Chain verification detects altered records and broken links.
An unsigned local chain cannot establish whether its entire contents were replaced or its tail was removed.
Signed checkpoints exist in source when evidence recording is enabled.
They help detect replacement or truncation before the signed head, but cannot establish integrity after that checkpoint.

**[ST-T-02]** An agent modifies policy.
The designed trust model loads trusted layers and lets a workspace layer only restrict.
The current CLI instead loads one operator-controlled workspace file; separate trust-layer loading is unimplemented.

**[ST-T-03]** An agent modifies Shiin's own state.
Shiin's state is a built-in protected resource that no policy can make allowable.

### Repudiation

**[ST-R-01]** An agent denies it performed an action.
Every decision is recorded in the audit log with a stable id.

### Information disclosure

**[ST-I-01]** An agent learns policy structure to probe for a bypass.
Agent-facing messages never reveal policy structure, error codes, or paths.

**[ST-I-02]** An agent reads secret values through logs.
Logs never contain secret values; identifiers and digests are used instead.

**[ST-I-03]** An agent reads the audit log.
The audit log is a built-in protected resource.

### Denial of service

**[ST-D-01]** An agent floods Shiin with requests.
The engine is synchronous and bounded; the adapter enforces timeouts that produce `deny`.
The daemon rejects noncanonical workspace roots and oversized normalized actions before policy evaluation; focused tests cover both bounds.
Audit append verifies the full existing chain with a streaming reader, so prior event payloads no longer accumulate in memory before an authorization can complete.
Regression tests reject early, middle, late, and malformed-tail corruption without appending; this preserves ST-T-01 while reducing aggregate-log memory usage.
Individual oversized audit records and total verification latency still require bounds.

**[ST-D-02]** A malicious policy file hangs Shiin.
Policy compilation is bounded and fails closed.

### Elevation of privilege

**[ST-E-01]** An agent uses a grant beyond its scope.
A grant is bound to a digest or a scope and never overrides `deny`.

**[ST-E-02]** An agent disables Shiin.
Adapters declare their enforcement level honestly; hook-based controls can be disabled.

**[ST-E-03]** An agent runs code that bypasses Shiin.
Shiin cannot see what an allowed command does; a sandbox covers that gap.
The MCP proxy forwards a tool call only after `allow`; its unit test confirms that `deny` and `pending` are not forwarded.
Its process integration test also checks that daemon unavailability prevents downstream execution and that plaintext tool arguments are omitted from audit events.
The downstream HTTP transport retains that authorization path and requires an explicitly configured endpoint.
TLS is required outside literal loopback addresses; URL credentials, query strings, redirects, automatic HTTP retries, environment proxies, and expired-session reconnection are refused or disabled.
Endpoint and redirect regression tests cover these boundaries; real remote-server authentication and independent transport review remain open (ST-I-02, ST-T-02).
Provisioned MCP bearer credentials now come from an explicitly named environment variable and are sent only to an HTTPS endpoint with redirects disabled.
Sensitive header marking, temporary-buffer zeroization, bounded credential/CA input, and fixed setup errors reduce accidental disclosure; tests check invalid/plaintext configuration and an authenticated TLS fixture (ST-I-02).
An explicit operator CA can extend trust without disabling hostname or certificate checks.
OAuth acquisition/refresh, external authenticated-server validation, and independent transport review remain open.
The cooperative command wrapper binds the exact executable/argument list and starts a child only after a durable allow.
Its process tests cover pending, denial, damaged policy/audit state, unavailable services, literal argument forwarding, and child status propagation ([ADR-0013](../adr/0013-fail-closed-failure-semantics.md), [ADR-0030](../adr/0030-audit-durability.md)).
Commands launched outside the wrapper and the internals of allowed child processes remain outside its interception point.

## Out of scope

Shiin is not designed to resist:

- Operating-system level attacks against the developer's account.
- Hardware or firmware attacks.
- Supply-chain attacks against Shiin's dependencies, which are covered by the supply-chain policy.

## Current changes and independent review

The 2026-10-15 reconciliation strengthens existing controls without making a failure or unknown configuration more permissive.
AI assistance was used; this section is a review packet, not an independent review or security sign-off.
The [M0 external-review criterion](../project/roadmap.md#m0-specification) remains open.

| Threat | Change or evidence | Remaining review |
|---|---|---|
| ST-I-01 | Hook failure and decision messages omit internal reason codes, file paths, and error text; regression tests exercise both response protocols. | Verify messages and exit behavior in running hosts, including startup failures. |
| ST-I-02 | Audit summaries omit free text and raw host input; command and URL values become digests. | Review identifier/path exposure and the limits of hashing low-entropy values. |
| ST-T-01 | Audit wire records follow the versioned schema and an existing chain is verified before append. | Review incompatible old-log handling and the absence of signed anchors. |
| ST-D-01 | Existing timeout, panic, contention, and budget tests remain required; the performance command measures local latency. | Large-input/log bounds and cross-platform memory measurements remain incomplete. |
| ST-S-01 | Hook identities remain asserted and the identity stage passes through. | Review the lack of assurance enforcement before claiming identity protection. |
| ST-T-02 | Built-in protections cannot be granted by operator rules. | Review single-file policy loading against the designed trust layers. |
| ST-E-02 | Cursor and Codex pending decisions are denied rather than relying on unsupported or unverified holding behavior; Claude Code also denies a pending response carrying a daemon ticket instead of treating it as a native prompt. | The adapter regression test covers this branch. Check host configuration, disabling controls, and every supported host version. |

Additional topics for the reviewer are shell classification coverage, path-error and Unicode/case handling, task timestamp parsing, and policy matching for multiple effects.
The Unix approval state machine and grant consumption, receipt signature verification, and replay have source implementations with focused tests. Their security properties still require independent review; Windows approval transport and live-host behavior remain unvalidated.

The external reviewer should record their name, reviewed commit or diff, platform and host versions, findings, and resolution of each finding.
No reviewer has supplied that record yet.

## Local approval service changes

The capability gate now requires explicit grants for the request class and every effect class, and any matching capability refusal wins.
Missing, invalid, or ambiguous declarations cannot produce allow; capability grants alone do not authorize an action ([ADR-0011](../adr/0011-decision-values-and-combining.md), [ADR-0012](../adr/0012-evaluation-stage-order.md)).
Seven new golden cases cover capability enforcement and session/task rule selectors, and the signed replay suite exercises them as well.
The snapshot/engine version changes prevent older replay inputs from silently receiving new capability semantics.
Session and agent identifiers remain asserted; they are selectors, not attestation (ST-S-01, ST-E-01).
The daemon continues to replace a caller's task reference with its authoritative active task before matching rules.

The Rust client now exercises the real daemon contract over a local socket, including policy validation, non-allow decisions, and stale snapshots.
Unsupported decision versions are rejected before an integration can act on them; this tightens failure handling without changing policy decisions ([ADR-0013](../adr/0013-fail-closed-failure-semantics.md)).
Replay regression tests now exercise the signed/encrypted conformance corpus through the CLI's evaluation routine, plus input bindings, grant coverage, deny precedence, and what-if comparisons.
These tests use explicit fixture keys without accessing or replacing the operator's OS keyring.
The separate native key-store lifecycle test uses the same storage methods under a random test-only service namespace on hosted runners ([ADR-0032](../adr/0032-signing-key-storage.md)).
It checks key persistence, rotation and retained-key replay, plus refusal of corrupt, deleted, or mismatched entries without regenerating a selected key (ST-T-01, ST-I-02).
The production service name remains fixed, and ordinary tests never access an operator's key store.
The first-alpha dependency refresh selects native stores through `keyring-core` directly, replacing the Linux Zbus transport with the DBus Secret Service backend ([ADR-0032](../adr/0032-signing-key-storage.md)).
Service/username bindings, key fingerprints, error handling, and refusal of a corrupt selected key stay unchanged; there is no plaintext or mock fallback (ST-I-02, ST-T-01).
The native lifecycle test remains required on every supported operating system after this backend change.
This validates storage behavior, not isolation from unmediated same-user processes or independent security review.
The schema runtime feature now separates OS clock/randomness functions from pure wire types and deterministic timestamp formatting.
Adapters explicitly retain the runtime feature and its fail-closed randomness behavior; isolated engine builds omit it.
The WebAssembly check and dependency guard check this separation without changing an authorization outcome (ST-T-01, ST-D-01).
The separate fuzz workspace exercises action-request parsing/validation and shell/SQL classification with bounded inputs and memory.
Its authorization target also checks determinism, unattended behavior, and capability refusal invariants against arbitrary validated requests (ST-E-01).
The determinism check exposed unordered determining-rule identifiers in otherwise identical decision traces.
The compiler now sorts those identifiers before returning any policy outcome; a regression compares serialized decisions repeatedly for multiple matching allow, deny, and pending rules (ST-R-01).
This canonicalizes evidence without changing which rules match, denial precedence, or an authorization outcome ([ADR-0010](../adr/0010-pure-synchronous-engine.md)).
Miri interprets pure schema tests; these robustness checks supplement the existing fault-injection tests and do not constitute independent review (ST-T-02, ST-D-01).

The v0.3 source implementation addresses ST-E-01 and ST-E-02 by requiring an out-of-band operator credential for resolution, listing, and revocation.
The local transport authenticates the peer's Unix user identity; an asserted agent role is insufficient to administer approvals.
Tests reject missing or incorrect credentials and generated transition sequences check that a single-use grant cannot authorize twice.
The Rust client's socket integration also exercises missing/forged operator proof, changed-action retries, single-use consumption, reusable scopes, built-in denial precedence, and revocation through the real service.
It checks that those transitions are audited and operator credentials are absent from the log.
After a single-use grant is spent, a new pending retry now receives a fresh unresolved ticket instead of reusing the already resolved ticket.
The client lifecycle test exercises reapproval while retaining exact-action binding and denial precedence (ST-E-01).
Grant bindings include the session, task, workspace, policy snapshot, action type, and effects; only the fresh request correlation UUID is excluded.
Explicit denials are re-evaluated before grants and cannot be overridden.
Audit and state writes finish before an allow response.

The credential is generated in memory and shown only in a dedicated operator terminal.
Terminal allocation alone is not proof of a human; the separate credential and typed confirmation provide the operator authorization step.
Allowed code running outside interception still shares the operating-system account and is outside the cooperative adapter's enforcement boundary.
Review of this boundary, restart/crash behavior, state-file races, and Windows transport remains required.
