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
Ubuntu's `systemd-oomd` log confirms that memory pressure killed the VS Code application group, which also contained the build processes.
Reducing build concurrency and debug information was insufficient to prevent a recurrence, so compilation was stopped.
Remaining cached tests ran sequentially in separate systemd scopes limited to 256 MiB, with no swap allowance.
No repository build profile was changed.

Final results are recorded below.

| Check | Result |
|---|---|
| Formatting | Passed |
| Workspace Clippy, all targets, warnings denied | Passed |
| Cached test executables, one test thread | Passed: 74 tests, including the 19-vector conformance corpus |
| Full `cargo test --workspace` command and doctests | Interrupted during builds; not completed end to end |
| Documentation contract check | Passed: cached checker, 130 pages |
| Markdown lint | Passed |
| Spelling | Passed |
| Links | Passed; zero errors |
| Book build | Passed; Mermaid preprocessor reports a version mismatch warning |
| Dependency audit | Interrupted; not established |
| Rust 1.96 minimum-version check | Toolchain installed; not established locally |
| Optimized performance and component memory budgets | Not established locally |
| Live host interoperability and other platforms | Not established locally |

The initial parallel test run failed a timing assertion under load and exposed a Cursor fixture missing its required conversation identifier.
The fixture was corrected; the timing limit and fail-closed deadline were preserved.
All 27 hook tests, seven wire-contract tests, 39 unit tests, and the one conformance test passed from cached executables.
The conformance test checks all 19 vectors.
These results do not claim that the interrupted Cargo command or unexecuted doctests passed.
The performance command passed compilation lint checks, but was not executed; the hook timing smoke test is not a substitute for milestone performance measurements.

## Hosted validation follow-up

For commit `052fbe5`, [CI run 38798096421](https://github.com/BAGOMBEKA-JOB-DEV/shiin/actions/runs/38798096421) completed the full Linux and macOS test jobs, Rust 1.96 check, and dependency audit successfully.
The documentation workflow also passed.
The nonblocking Windows job failed because CRLF Markdown fences prevented the policy-example test from finding its three examples.
The test now normalizes line endings without skipping examples or weakening its assertions.
That fix awaits the next Windows run.

Hosted Linux and macOS release-latency jobs have been added to measure the existing budgets without rebuilding on the laptop.
Their results will be recorded after execution; component memory profiling and large-policy/log measurements remain open.

## Security review and milestone status

The [threat-model review packet](docs/security/threat-model.md#current-changes-and-independent-review) maps changes and tests to existing threat identifiers.
An outside reviewer still needs to record their identity, reviewed commit, findings, and resolutions.
The suggested account, `BAGOMBEKA-JOB-DEV`, authored PR #12 and therefore does not meet the outside-project criterion.
No outside reviewer or human sign-off has been obtained, so M0 remains open.

The hook tests execute the real Shiin binary with protocol payloads; they do not launch Claude Code, Codex, or Cursor.
Live host configuration, timeout/crash handling, pending behavior, and host-version coverage still need validation before claiming the v0.1/v0.2 exit criteria are complete.
Cross-platform latency, component memory limits, large audit histories, and the intent-binding evaluation plan also remain open.

Maintainer review should cover asserted identity, incomplete trust-layer loading, shell classification, path-error and Unicode/case handling, task timestamps, multi-effect policy matching, audit redaction, and old-log handling.
The platform support page and ADR-0040 also need a maintainer decision about their differing support tiers.
These limitations were recorded rather than resolved through silent changes to accepted decisions.

The PR is a draft pending human security review, factual verification of third-party claims, and the human Developer Certificate of Origin sign-off required by the contribution policy.
