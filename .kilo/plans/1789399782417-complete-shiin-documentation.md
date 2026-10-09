# Plan: Complete All Shiin Documentation

## Context

**Project:** Shiin — agent-agnostic authorization layer for AI agent actions. Pre-alpha specification phase (M0). Rust workspace (edition 2024, toolchain 1.98.1).

**Current state:** Only `xtask` crate exists, providing `cargo xtask docs-check` (a Markdown/doc validator). `docs/` has 22 Markdown files but references ~62 more across `spec/`, `security/`, `architecture/`, `engineering/`, `reference/`, `project/`, `concepts/`, `getting-started/`, `guides/`, `rfcs/`. ADR index lists 42 ADRs but only 0001–0002 exist as files. README.md at repo root has committed Git merge-conflict markers.

**Goal:** All documentation pages exist, are internally consistent, and pass every check in `docs/project/docs-tooling-and-ci.md`.

---

## Constraints

- **docs-check** validates each page: metadata comment on line 1, blank line 2, `# Title` line 3, blank line 4, banner line 5–6. Full rules in `docs/project/docs-style-guide.md`.
- **ADRs** follow MADR 4 template (`docs/adr/template.md`). Titles/statuses from `docs/adr/README.md` index. Content derived from references in existing docs + ADR's stated purpose.
- **Requirement IDs** defined only on `kind=spec` pages. Prefixes owned by specific spec files (from `xtask/src/docs_check/ids.rs` and `docs-style-guide.md` §"Requirement ID prefixes"). Format: `PREFIX-NNN` (3 digits).
- **Threat IDs** defined only in `security/threat-model.md`. Format: `ST-<S|T|R|I|D|E>-<nn>`.
- **Former name** "latch" allowed only in `adr/0003-project-name-shiin.md` and `overview/landscape.md`.
- **README.md at repo root** is NOT under `docs/` and not checked by docs-check. It is a GitHub-facing project README.
- No network access during implementation. All content reconstructsible from existing docs.

---

## Decisions

1. **README.md resolution:** Keep the HEAD/project-README side (lines 2–128 of current file), discard the incoming docs-index side (the `=======` block onward). `docs/README.md` already exists as the docs index.
2. **Page metadata per SUMMARY.md entry:** Derive `kind`, `status`, `implementation`, `milestone`, `reviewed` from each page's role — see per-page table below.
3. **Schemas** go in `docs/spec/schemas/`. Naming: `<name>.v<major>.schema.json` with `$id: urn:shiin:schema:<name>:v<major>`. Required: `common.v1`, `test-vector.v1`, and one schema per spec that has validated JSON examples.
4. **Test vectors** go in `docs/spec/test-vectors/`, validated against `test-vector.v1.schema.json`. Encode golden use cases from `docs/overview/use-cases.md` (UC-01 through UC-09).
5. **Order of creation** is chosen so each phase's cross-references can resolve: ADRs → specs → security → schemas/test-vectors → remaining docs → README fix.

---

## Phase 0 — ADRs (0003–0042)

Create 40 ADR files. Title and status come from `docs/adr/README.md` index. Content follows `docs/adr/0001-record-architecture-decisions.md` and `docs/adr/0002-rust-for-product-and-tooling.md` format. Content for each derived from: its title, references in existing docs, and the design described in vision/use-cases/glossary.

ADR index with required metadata:

| # | Title | Status | Key references in existing docs |
|---|---|---|---|
| 0003 | Project name: Shiin | accepted | `terms.rs` FORMER_NAME_PAGES |
| 0004 | Apache-2.0 license and DCO | accepted | vision.md §License, README |
| 0005 | Governance model | accepted | README → GOVERNANCE.md |
| 0006 | Vulnerability reporting | accepted | README → SECURITY.md |
| 0007 | No telemetry | accepted | vision.md principle 9, use-cases UC-08 |
| 0008 | mdBook documentation | accepted | ADR-0001, docs-tooling-and-ci.md |
| 0009 | Deterministic enforcement | accepted | vision.md principle 2, landscape |
| 0010 | Pure synchronous engine | accepted | how-shiin-works.md, glossary engine |
| 0011 | Decision values and combining | accepted | glossary decision/deny/pending, vision principle 4 |
| 0012 | Evaluation stage order | accepted | how-shiin-works.md §five stages, glossary stage |
| 0013 | Fail-closed failure semantics | accepted | coding-standards.md, vision principle 5, glossary fail-closed |
| 0014 | Cedar policy engine with TOML front-end | proposed | landscape.md, vision, roadmap M0 exit |
| 0015 | Policy layering and trust | accepted | glossary policy-layer, how-shiin-works examples |
| 0016 | Built-in protected resources | accepted | glossary, vision principle 9 (keys/audit) |
| 0017 | JSON wire format and schemas | accepted | workspace-and-crates, glossary |
| 0018 | Conservative action classification | accepted | glossary classifier/opaque, use-cases UC-02 |
| 0019 | Path canonicalization | proposed | how-shiin-works, use-cases UC-02 |
| 0020 | Structured intent binding | accepted | glossary intent-binding/task-declaration, use-cases UC-04 |
| 0021 | Discrete risk signals | accepted | glossary risk-signal, how-shiin-works, use-cases |
| 0022 | Local-first, daemonless deployment | accepted | roadmap v0.1/v0.3, how-shiin-works |
| 0023 | JSON-RPC local API | accepted | roadmap v0.3/v0.5, glossary |
| 0024 | Tokio, confined async runtime | accepted | workspace-and-crates, roadmap v0.3 |
| 0025 | Adapter roadmap | accepted | landscape.md, how-shiin-works |
| 0026 | Adapter extensibility | accepted | roadmap v0.5/v0.6 |
| 0027 | Approval resolutions and grants | accepted | glossary grant/resolution, use-cases UC-05 |
| 0028 | Deny-with-ticket for non-waiting hosts | accepted | glossary deny-with-ticket, vision |
| 0029 | Hash-chained JSONL audit log | accepted | glossary audit-log/hash-chain, use-cases UC-06 |
| 0030 | Audit durability | accepted | error-handling.md, glossary |
| 0031 | DSSE and Ed25519 receipts | accepted | glossary receipt, use-cases UC-06, roadmap v0.4 |
| 0032 | Signing key storage | accepted | glossary, vision |
| 0033 | Enforcement levels | accepted | glossary enforcement-level, landscape, how-shiin-works |
| 0034 | Edition, MSRV, and toolchain | accepted | toolchain-and-msrv.md, coding-standards.md |
| 0035 | Workspace layout | accepted | workspace-and-crates.md |
| 0036 | Error handling | accepted | error-handling.md |
| 0037 | Unsafe code policy | accepted | unsafe-policy.md |
| 0038 | Supply chain policy | accepted | workspace-and-crates.md §dependencies |
| 0039 | Release tooling | accepted | release-engineering.md |
| 0040 | Platform support tiers | accepted | toolchain-and-msrv, platform-support.md |
| 0041 | Remote approver transport | deferred (v0.6) | roadmap v0.6 |
| 0042 | Language bindings | accepted | ADR-0002, roadmap after-1.0 |

**Validation:** All ADR titles start with `# ADR-NNNN: `. All listed in `docs/adr/README.md` index (already done). All `ADR-NNNN` mentions in other pages now resolve.

## Phase 1 — Specification Pages (spec/)

Create 15 spec pages. Each must match the metadata/banner rules for `kind=spec`. Define requirement IDs (`**[PREFIX-NNN]**`) where specified by existing docs. The `ids.rs` check enforces that every `PREFIX-NNN` reference resolves to exactly one definition on the correct owning page, and that the prefix is known.

Page-level ownership (from `xtask/src/docs_check/ids.rs`):

| File | Prefix | Key content from existing docs |
|---|---|---|
| spec/README.md | — | Lists all specs, versions, test vectors. From glossed "Specified" list |
| spec/action-types.md | AT | Dotted identifiers (fs.read, shell.exec). Glossary: "Action type" entry, use-cases |
| spec/action-request.md | AR | Normalized request: agent identity, action type, resources, effects, task. Glossary + how-shiin-works §path-of-one-action step 3 |
| spec/decision.md | DEC | Values allow/deny/pending, reason codes, combining. Glossary decision/deny/pending/default-decision. ADR-0011 |
| spec/evaluation.md | EVAL | Five stages, combine rules, policy snapshot input. how-shiin-works §five-stages. ADR-0010, ADR-0012 |
| spec/identity.md | ID | Agent identity, assurance levels. Glossary agent-identity/assurance-level |
| spec/policy-model.md | POL | Rules, capabilities, layers, protected resources. Glossary policy/policy-layer/built-in-protected. ADR-0015, ADR-0016 |
| spec/policy-syntax.md | SYN | TOML → Cedar. how-shiin-works examples, glossary. ADR-0014 |
| spec/intent-binding.md | INT | Task declarations, constraints, counters. Glossary task-declaration/constraint/intent-binding. ADR-0020 |
| spec/risk-signals.md | RISK | Discrete signals: secret_path, outside_workspace, bulk_delete, opaque_command, destructive_vcs. Glossary + use-cases |
| spec/approval-protocol.md | APR | Approval requests, resolutions, grants, deny-with-ticket. Glossary approver/approval-request/grant/resolution. ADR-0027, ADR-0028 |
| spec/audit-events.md | AUD | Event types, hash chain, checkpoints. Glossary audit-event/audit-log/checkpoint. ADR-0029, ADR-0030 |
| spec/receipts.md | RCPT | DSSE, Ed25519, verification. Glossary receipt/receipt-verification. ADR-0031 |
| spec/adapter-contract.md | ADP | Host protocol, hook interface, enforcement levels. Glossary adapter/hook. ADR-0025 |
| spec/local-api.md | API | JSON-RPC over daemon. Glossary. ADR-0022, ADR-0023 |
| spec/test-vectors/README.md | — | Test vector format, encodes use cases |

**Requirements for passing docs-check:**
- Every spec page defines its own requirement IDs (e.g., `**[AR-001]**` in action-request.md).
- No requirement ID is referenced (`<PREFIX>-<NNN>` pattern) unless defined. The `docs-style-guide.md` mentions `EVAL-007` as an example — that must be defined in `spec/evaluation.md`.
- Each spec ends with a "Revision history" section and states its version.

## Phase 2 — Security Pages (security/)

Create 7 pages. All are `kind=reference` or `kind=spec` with appropriate metadata.

| File | Kind | Key content |
|---|---|---|
| security/threat-model.md | spec | STRIDE threats `ST-X-NN`, defined here only. Referenced by glossary. ADR-0013 |
| security/security-model.md | spec | SEC-prefixed requirements. Failure semantics, deny-reason oracles. |
| security/enforcement-levels.md | reference | L1/L2/L3 definitions. Glossary enforcement-level. ADR-0033 |
| security/cryptography.md | reference | Hash functions, signing, checkpoints. Glossary checkpoint/receipt |
| security/vulnerability-disclosure.md | policy | Reporting process, SECURITY.md. ADR-0006 |
| security/hardening.md | explanation | Sandboxing, least privilege, reducing blast radius |
| security/verifying-releases.md | reference | Binary signing, SBOM, checksums |

**Threat IDs:** Enumerate STRIDE threats covering the design. Examples from existing docs: deny-reason oracles (error-handling.md), prompt injection (vision), excessive agency (vision LLM06). Each `ST-X-NN` defined once in threat-model.md; every mention elsewhere resolves.

## Phase 3 — Architecture Pages (architecture/)

Create 4 pages (all `kind=explanation`):

| File | Key content from existing docs |
|---|---|
| architecture/overview.md | Component-to-crate mapping. Existing docs reference from roadmap/vision |
| architecture/decision-pipeline.md | The five-stage pipeline. how-shiin-works §five-stages |
| architecture/deployment-models.md | Local-first, daemonless, team server (post-1.0). ADR-0022, vision |
| architecture/state-and-storage.md | Audit log location, policy cache, grant store. error-handling.md, glossary |

## Phase 4 — Remaining Engineering Pages (engineering/)

Create 6 pages (all `kind=policy`):

| File | Key content |
|---|---|
| engineering/async-and-concurrency.md | Tokio confined model. ADR-0024, workspace-and-crates |
| engineering/dependencies-and-supply-chain.md | Dep admission criteria. ADR-0038, unsafe-policy.md |
| engineering/performance-budgets.md | Latency/memory budgets. Coding-standards, roadmap, vision success |
| engineering/ci.md | CI pipeline, MSRV check, clippy `-D warnings`. toolchain-and-msrv, coding-standards |
| engineering/release-engineering.md | Releases, SBOM, signing. ADR-0039, roadmap v0.1/v0.4 |
| engineering/security-sensitive-changes.md | Review gates for security-critical changes. unsafe-policy.md |

## Phase 5 — Reference Pages (reference/)

Create 4 pages. `cli.md` and `error-codes.md` are `kind=reference` with `implementation=none` → use WARNING banner "Design intent: not implemented".

| File | Key content |
|---|---|
| reference/cli.md | CLI subcommands from roadmap: init, check, explain, policy validate/test, hook, task, pending, approve, log, verify, replay, doctor. CLI reference |
| reference/configuration.md | `.shiin/` layout, config file format |
| reference/error-codes.md | Stable error codes `SHIIN-XXX`. Error-handling.md §Error-codes |
| reference/platform-support.md | Tier 1/2/3 platforms. ADR-0040, toolchain-and-msrv |

## Phase 6 — Remaining Project Pages (project/)

Create 3 pages (all `kind=policy`):

| File | Key content |
|---|---|
| project/privacy-and-telemetry.md | No telemetry design. ADR-0007, vision principle 9 |
| project/versioning-and-stability.md | Versioning policy, SemVer, API stability. vision §Stability |
| project/maintainer-handbook.md | Maintainer duties, releases, review. ADR-0005 |

## Phase 7 — Concepts Pages (concepts/)

Create 11 pages (all `kind=explanation`). Content from glossary entries and how-shiin-works:

| File | Source material |
|---|---|
| concepts/action-requests.md | Glossary action-request, how-shiin-works flow |
| concepts/decisions.md | Glossary decision, ADR-0011 |
| concepts/agents-and-identity.md | Glossary agent/agent-identity/assurance-level, spec/identity |
| concepts/capabilities.md | Glossary capability, policy-model |
| concepts/policies.md | Glossary policy/rule/policy-layer, policy-model |
| concepts/tasks-and-intent.md | Glossary task-declaration/intent-binding, intent-binding spec |
| concepts/risk-signals.md | Glossary risk-signal, risk-signals spec, use-cases |
| concepts/approvals.md | Glossary approver/approval-request/grant/resolution, approval-protocol |
| concepts/adapters.md | Glossary adapter/hook/enforcement-level, adapter-contract |
| concepts/audit-and-evidence.md | Glossary audit-log/audit-event/receipt/checkpoint, audit-events/receipts |
| concepts/limitations.md | vision §Non-goals, how-shiin-works §limitations |

## Phase 8 — Getting-Started Tutorials (getting-started/)

Create 3 pages (all `kind=tutorial`, `implementation=none` → WARNING banner):

| File | Key content |
|---|---|
| getting-started/installation.md | Install instructions, rustup, cargo-binstall. toolchain-and-msrv, docs-tooling-and-ci |
| getting-started/quickstart-claude-code.md | End-to-end example with Claude Code. use-cases UC-02, how-shiin-works |
| getting-started/first-policy.md | Writing a policy.toml. policy-syntax, writing-policies guide |

## Phase 9 — Guides and Integrations (guides/)

Create 12 pages. Guides are `kind=how-to` with `implementation=none` → WARNING banner. Integrations reference host hook behaviors (verified dates from landscape.md):

| File | Key content |
|---|---|
| guides/writing-policies.md | Policy TOML syntax, examples. policy-syntax |
| guides/using-shiin-with-sandboxes.md | Layering with sandboxes. vision §complements-sandboxes, limitations |
| guides/declaring-tasks.md | Task declarations. intent-binding |
| guides/testing-policies.md | `shiin policy test`. policy-syntax, use-cases UC-07 |
| guides/handling-approvals.md | Approval protocol usage. approval-protocol |
| guides/audit-replay-and-receipts.md | `shiin log`, `verify`, `replay`. audit-events, receipts |
| guides/integrations/claude-code.md | Claude hooks (PreToolUse, PermissionRequest). landscape.md verified |
| guides/integrations/codex.md | Codex hooks. landscape.md verified |
| guides/integrations/cursor.md | Cursor hooks (preToolUse etc.). landscape.md verified |
| guides/integrations/mcp-proxy.md | MCP proxy design. ADR-0025, use-cases UC-08 |
| guides/integrations/custom-agents.md | Adapter contract for custom agents |
| guides/integrations/aider-sandboxed.md | Aider in sandbox |

## Phase 10 — RFC Pages (rfcs/)

Create 2 pages:

| File | Kind | Content |
|---|---|---|
| rfcs/README.md | policy | RFC process: how to propose, review, accept. ADR-0001 |
| rfcs/0000-template.md | policy | Template for RFC documents |

## Phase 11 — JSON Schemas and Test Vectors

Create schemas in `docs/spec/schemas/`. Each must have `"$schema": "https://json-schema.org/draft/2020-12/schema"` and `$id: urn:shiin:schema:<name>:v1`.

**Schemas to create:**
- `common.v1.schema.json` — shared definitions (uuid, etc.), referenced by `$id`
- `test-vector.v1.schema.json` — validates test vector files
- One schema per spec with JSON examples: `action-request.v1.schema.json`, `decision.v1.schema.json`, `evaluation.v1.schema.json`, `policy-model.v1.schema.json`, `intent-binding.v1.schema.json`, `approval-protocol.v1.schema.json`, `audit-event.v1.schema.json`, `receipt.v1.schema.json`

**Test vectors** in `docs/spec/test-vectors/`:
- `README.md` — format description
- One `.json` file per golden use case (UC-01 through UC-09), validated against `test-vector.v1.schema.json`

**Wire format derivation:** From `docs/glossary.md`, `how-shiin-works.md`, and concept page descriptions. Action request = agent identity + action type + effects + task. Decision = allow/deny/pending + reason code. Effects = (path + operation, confidence exact/inferred/opaque).

## Phase 12 — README.md Merge-Conflict Resolution

Remove the `<<<<<<< HEAD` / `=======` / `>>>>>>>` markers and the incoming docs-index content (beginning with the `<!-- shiin-doc: ... -->` comment). Keep the project README (the HEAD side). `docs/README.md` already holds the docs index separately.

## Phase 13 — Validation

Run each check from `docs/project/docs-tooling-and-ci.md` §"Running the checks", in order:

```
cargo xtask docs-check
rumdl check .
typos
lychee --config lychee.toml ./*.md docs
mdbook build docs
```

**Fix iteratively:** docs-check reports all failures with file:line locations. Fix them until it prints "docs-check: N pages OK". Then rumdl, typos, lychee, mdbook.

**Cargo note:** Cargo is at `~/.cargo/bin` on this machine. If not on PATH, prefix: `PATH="$HOME/.cargo/bin:$PATH" cargo xtask docs-check`. First run compiles dependencies (may take several minutes).

---

## Deliverables Checklist

- [ ] 40 ADR files (0003–0042) with correct titles, statuses, MADR format
- [ ] 15 spec pages with requirement IDs covering all prefixes (AT, AR, DEC, EVAL, ID, POL, SYN, INT, RISK, APR, AUD, RCPT, ADP, API)
- [ ] 7 security pages with threat IDs (ST-X-NN) and SEC-prefixed requirements
- [ ] 4 architecture pages
- [ ] 6 remaining engineering pages
- [ ] 4 reference pages
- [ ] 3 remaining project pages
- [ ] 11 concept pages
- [ ] 3 getting-started tutorial pages
- [ ] 12 guides + integrations pages
- [ ] 2 RFC pages
- [ ] 8+ JSON schema files in `docs/spec/schemas/`
- [ ] 9+ test vector files in `docs/spec/test-vectors/`
- [ ] README.md merge-conflict resolved
- [ ] `cargo xtask docs-check` passes
- [ ] `rumdl check .` passes
- [ ] `typos` passes
- [ ] `lychee` passes (offline mode: local links only)
- [ ] `mdbook build docs` succeeds
