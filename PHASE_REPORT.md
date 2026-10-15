# Contract reconciliation and validation report

Prepared on 2026-10-15 with AI assistance.
This report records implementation evidence and remaining milestone criteria; it is not an independent security review.

## Project state

Shiin is a pre-alpha authorization layer with seven product crates and a Rust `xtask` tool.
The current source build normalizes hook input, compiles TOML policy to Cedar, evaluates five stages, applies experimental task constraints, and records hash-chained audit events.
It has no published release.
Identity and capability stages remain pass-through, and the CLI loads one operator-controlled policy file rather than the complete designed trust hierarchy.

The documentation now separates this implementation from planned capabilities.
The roadmap remains approvals and grants in v0.3, signed evidence and replay in v0.4, and MCP/API/client integration in v0.5.
No crates for those future milestones were added.

## Changes in this phase

- Corrected stale implementation banners and source-build instructions across the documentation.
- Reconciled draft TOML policy syntax and audit schemas with emitted values, advancing those drafts to `v1-draft.2`.
- Corrected UUID examples and generated request identifiers to the accepted UUIDv4 requirement.
- Added validation of required request values at CLI boundaries and rejection of unknown risk configuration.
- Made serialized decisions conform to their schema, with generic messages and a five-stage operator trace.
- Removed internal rule/error details from hook responses and conservatively mapped Cursor pending decisions to denial.
- Redacted command and URL effect values in audit summaries, retained a digest of the original normalized body, and rejected extension of corrupted or unsupported logs.
- Added wire-schema, policy-example, and hook regression tests, plus a Rust performance command.

The audit format change is incompatible with earlier pre-alpha logs.
Preserve old logs separately; this change supplies no migration and does not delete evidence.
Audit summaries cannot support exact replay, and unsigned hash chains cannot prove that their entire history has not been replaced or truncated.

## Validation environment and results

Validation ran on a local Linux x86_64 laptop with approximately 3.6 GiB RAM and Rust 1.98.1.
Native build dependencies and the required Rust documentation tools were installed.
Editor closures interrupted earlier builds; subsequent work used one build process, lower scheduling priority, and reduced command-binary debug information.
No repository build profile was changed.

Final results are recorded below before this report is submitted.

| Check | Result |
|---|---|
| Formatting | Pending final run |
| Workspace Clippy, all targets, warnings denied | Passed |
| Workspace tests, one test thread | Pending final run |
| Documentation contract check | Pending final run |
| Markdown lint | Pending final run |
| Spelling | Pending final run |
| Links | Passed: 1,581 checked references, zero errors |
| Book build | Pending final run |
| Dependency audit | Interrupted; not established |
| Rust 1.96 minimum-version check | Toolchain installed; not established locally |
| Optimized performance and component memory budgets | Not established locally |
| Live host interoperability and other platforms | Not established locally |

The initial parallel test run failed a timing assertion under load and exposed a Cursor fixture missing its required conversation identifier.
The fixture was corrected; the timing limit and fail-closed deadline were preserved.
The final test result above supersedes that run only for the stated local environment.

## Security review and milestone status

The [threat-model review packet](docs/security/threat-model.md#current-changes-and-independent-review) maps changes and tests to existing threat identifiers.
An outside reviewer still needs to record their identity, reviewed commit, findings, and resolutions.
No outside reviewer or human sign-off has been obtained, so M0 remains open.

The hook tests execute the real Shiin binary with protocol payloads; they do not launch Claude Code, Codex, or Cursor.
Live host configuration, timeout/crash handling, pending behavior, and host-version coverage still need validation before claiming the v0.1/v0.2 exit criteria are complete.
Cross-platform latency, component memory limits, large audit histories, and the intent-binding evaluation plan also remain open.

Maintainer review should cover asserted identity, incomplete trust-layer loading, shell classification, path-error and Unicode/case handling, task timestamps, multi-effect policy matching, audit redaction, and old-log handling.
The platform support page and ADR-0040 also need a maintainer decision about their differing support tiers.
These limitations were recorded rather than resolved through silent changes to accepted decisions.

The PR is a draft pending human security review, factual verification of third-party claims, and the human Developer Certificate of Origin sign-off required by the contribution policy.
