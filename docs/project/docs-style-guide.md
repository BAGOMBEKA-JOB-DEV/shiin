<!-- shiin-doc: kind=policy status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# Documentation style guide

> [!NOTE]
> **Project policy: accepted.**
> Every page under `docs/` follows this guide.
> Rules marked **(checked)** are enforced by `cargo xtask docs-check`.

Shiin's documentation is written before its implementation.
This guide keeps a large set of pages consistent, honest, and verifiable
so that readers, contributors, and security reviewers can trust what they read.

## The honesty rule

A reader must never mistake a design for a working feature.

- Never claim that something exists today (that it can be installed, runs, is tested, or is supported)
  unless the page's `implementation` field is `partial` or `complete`.
- Pages that describe designed behavior may use the present tense ("the engine evaluates stages in order")
  because their banner states that no implementation exists.
- When a page mixes built and planned behavior, mark each planned item inline, for example `(planned: v0.3)`.
- The pull request that ships a feature updates the affected pages' `implementation` and `milestone` fields and their banners.
  **(checked: the banner must match the metadata)**

## Document kinds

Each page has exactly one kind.
Kinds follow the [Diátaxis](https://diataxis.fr/) framework, extended with project-specific kinds.

| Kind | Purpose | Usual location |
|---|---|---|
| `index` | Landing page that orients readers | `docs/README.md` |
| `explanation` | Why the design is the way it is | `overview/`, `concepts/`, `architecture/` |
| `spec` | Precise, normative contracts that implementations are checked against | `spec/`, `security/security-model.md` |
| `reference` | Facts to look up | `reference/`, `glossary.md` |
| `how-to` | Steps to accomplish a goal | `guides/` |
| `tutorial` | A guided first experience | `getting-started/` |
| `adr` | One architecture decision | `adr/` |
| `rfc` | The proposal process and its template | `rfcs/` |
| `policy` | Rules the project and its contributors follow | `project/`, `engineering/`, `security/vulnerability-disclosure.md` |

## Page metadata (checked)

Line 1 of every page under `docs/`, except `SUMMARY.md`, is exactly one HTML comment:

```text
<!-- shiin-doc: kind=<kind> status=<status> implementation=<implementation> milestone=<milestone> reviewed=<YYYY-MM-DD> -->
```

The comment is invisible on GitHub and in the built book, so no front-matter preprocessor is needed.
Fields appear in this order, separated by single spaces.

| Field | Allowed values | Meaning |
|---|---|---|
| `kind` | see [Document kinds](#document-kinds) | What sort of page this is |
| `status` | `draft`, `proposed`, `accepted`, `deferred`, `superseded`, `deprecated` | Where the content is in its review lifecycle |
| `implementation` | `none`, `partial`, `complete`, `n/a` | How much of the described behavior exists in released code; `n/a` for process pages |
| `milestone` | `m0`, `v0.1` through `v0.6`, `v1.0`, `post-1.0`, `n/a` | The roadmap milestone the content targets; see [Roadmap](roadmap.md) |
| `reviewed` | ISO 8601 date | When a maintainer last confirmed the page is accurate |

`spec`, `tutorial`, and `how-to` pages must not use `implementation=n/a`.

## Title and banner (checked)

After the metadata comment and a blank line comes the page title as a level-1 heading,
then a blank line, then a banner written as a GitHub-style admonition.
mdBook and GitHub both render these admonitions.

```markdown
<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-09-14 -->

# Action request

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
```

The first banner line after the admonition marker must begin with the label shown below.
The rest of the banner is free text.

| Metadata | Admonition | Banner label |
|---|---|---|
| `kind=index` | `[!NOTE]` | `**Project status: pre-alpha.**` |
| `kind=spec` | `[!NOTE]` | `**Specification: <status>.**` |
| `kind=explanation` | `[!NOTE]` | `**Design document: <status>.**` |
| `kind=adr` | `[!NOTE]` | `**Architecture decision: <status>.**` |
| `kind=rfc` | `[!NOTE]` | `**RFC: <status>.**` |
| `kind=policy` | `[!NOTE]` | `**Project policy: <status>.**` |
| `kind` is `tutorial`, `how-to`, or `reference`, with `implementation=none` | `[!WARNING]` | `**Design intent: not implemented.**` |
| `kind` is `tutorial` or `how-to`, implemented | `[!NOTE]` | `**Guide: <status>.**` |
| `kind=reference`, implemented or `n/a` | `[!NOTE]` | `**Reference: <status>.**` |

Recommended wording for design-intent pages:
"Planned for v0.1. Nothing on this page works yet. It describes the intended behavior so it can be reviewed before it is built."

## Files, folders, and headings

- File and folder names are lowercase `kebab-case` with the `.md` extension.
- Every page appears in [`SUMMARY.md`](../SUMMARY.md), and every `SUMMARY.md` entry points to an existing page. **(checked)**
- Headings use sentence case: "Decision pipeline", not "Decision Pipeline".
- A page has one level-1 heading.
  Headings must be unique within a page, because anchors are derived from heading text.
- Do not use heading ID attributes (`{#id}`) or definition lists; GitHub renders neither.

## Writing style

- Use American English and the serial (Oxford) comma.
- Write one sentence per line (semantic line breaks).
  Rendering is unaffected, and diffs show exactly which sentence changed.
- Prefer short, direct sentences. Put the conclusion first.
- Address the reader as "you" in tutorials and how-to guides.
  Specifications describe roles ("the adapter", "the approver") and never "you".
- Link a term to the [glossary](../glossary.md) the first time it appears on a page.
- Never use humor, marketing superlatives, or unqualified security claims ("unbreakable", "fully secure").

## Terminology (checked in part)

[`glossary.md`](../glossary.md) is the only place terms are defined.
It also contains the table of avoided terms and their replacements.
The most important rules:

- Decision values are `allow`, `deny`, and `pending`.
  Use "ask" only when naming a host's native mechanism, such as Claude Code's `ask` permission decision.
  **(checked: `block` in code formatting is rejected)**
- A human *approves*; the resulting decision is *allow*.
- Shiin's unit is the *action*. "Tool call" is the host's term and appears only on host-specific pages.
- Say *capability* or *rule*, not "permission", for Shiin concepts.
- Shiin is an *authorization layer*.
  It is never described as a firewall, guardrail, or sandbox, except when contrasting it with those tools.
  **(checked: "Shiin is a firewall" and similar phrasings are rejected)**
- Write "allowlist" and "denylist". **(checked)**
- "Shiin" is the project. `shiin` is the command-line binary. `shiind` is the daemon.
- The former working name of the project appears only in [ADR-0003](../adr/0003-project-name-shiin.md) and the [landscape](../overview/landscape.md). **(checked)**

## Specifications

Specifications are the contracts that code, tests, and conformance suites are checked against.

- Only `kind=spec` pages use requirement keywords. **(checked)**
  Each specification includes this sentence in its conventions section:
  "The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals."
- Every normative requirement starts with a bold, bracketed requirement ID: `**[AR-001]**`.
  IDs are unique across all documentation, are defined only on `kind=spec` pages, and are never renumbered or reused.
  A withdrawn requirement keeps its ID with the text "Withdrawn." **(checked)**
- Any mention of a requirement ID elsewhere must refer to a defined ID. **(checked)**
- Each specification states its version (for example `shiin.action-request/v1-draft.1`) near the top
  and ends with a "Revision history" section.
- Every JSON example that claims to be valid is validated against its schema (see [Validated examples](#validated-examples)).

### Requirement ID prefixes

| Prefix | Owning specification |
|---|---|
| `AT` | [Action types](../spec/action-types.md) |
| `AR` | [Action request](../spec/action-request.md) |
| `DEC` | [Decision](../spec/decision.md) |
| `EVAL` | [Evaluation](../spec/evaluation.md) |
| `ID` | [Identity](../spec/identity.md) |
| `POL` | [Policy model](../spec/policy-model.md) |
| `SYN` | [Policy syntax](../spec/policy-syntax.md) |
| `INT` | [Intent binding](../spec/intent-binding.md) |
| `RISK` | [Risk signals](../spec/risk-signals.md) |
| `APR` | [Approval protocol](../spec/approval-protocol.md) |
| `AUD` | [Audit events](../spec/audit-events.md) |
| `RCPT` | [Receipts](../spec/receipts.md) |
| `ADP` | [Adapter contract](../spec/adapter-contract.md) |
| `API` | [Local API](../spec/local-api.md) |
| `SEC` | [Security model](../security/security-model.md) |

Requirement IDs have three digits after the prefix (`EVAL-007`).

### Threat IDs

Threats are defined in the [threat model](../security/threat-model.md) as `**[ST-<category>-<nn>]**`,
where the category is a STRIDE letter (`S`, `T`, `R`, `I`, `D`, or `E`) and `nn` has two digits,
for example `**[ST-T-03]**`.
Any mention of a threat ID must refer to a defined threat. **(checked)**

## JSON Schemas

- Schemas live in `docs/spec/schemas/` and are named `<name>.v<major>.schema.json`.
- Every schema declares `"$schema": "https://json-schema.org/draft/2020-12/schema"`. **(checked)**
- Every schema has an `$id` of the form `urn:shiin:schema:<name>:v<major>`, for example `urn:shiin:schema:action-request:v1`.
  URNs keep schema identity independent of any web domain. **(checked)**
- Shared definitions live in `common.v1.schema.json` and are referenced by `$id`, for example `"$ref": "urn:shiin:schema:common:v1#/$defs/uuid"`.
- Test vectors live in `docs/spec/test-vectors/` and are validated against `test-vector.v1.schema.json`. **(checked)**

## Validated examples

To have a JSON example validated, put a marker comment on the line directly before its code fence.
The path is relative to `docs/`.

````markdown
<!-- validate: spec/schemas/decision.v1.schema.json -->
```json
{ "...": "..." }
```
````

Use `<!-- validate-fail: <schema path> -->` for an example that is deliberately invalid;
the check fails if such an example validates. **(checked)**

## Code blocks and example data

- Always tag fenced code blocks with a language:
  `toml`, `json`, `rust`, `console` (a shell session with `$` prompts), `text`, `mermaid`, or `cedar`.
- Use domains reserved by RFC 2606 (`example.com`, `example.org`, `*.example`) for hosts.
- Use fake secrets that contain the word `EXAMPLE`, such as `sk-EXAMPLE-0000000000000000`.
  Never paste a real-looking credential, even an expired one.
- Use `/home/dev/project` (Linux), `/Users/dev/project` (macOS), and `C:\Users\dev\project` (Windows) as example paths.

## Diagrams

- Use Mermaid in fenced `mermaid` blocks only. GitHub renders them natively, and the book renders them through `mdbook-mermaid`.
- Label nodes with glossary terms.
- Do not set custom colors or themes; they break dark mode.
- Put a one-sentence summary of the diagram in the text before it, so the page makes sense without the image.

## Links and cross-references

- Link between pages under `docs/` with relative paths that include the `.md` extension.
- Link to files outside `docs/` (such as `SECURITY.md`) with absolute GitHub URLs,
  because the built book does not contain them.
- Refer to decisions as `ADR-NNNN` with a link, for example [ADR-0014](../adr/0014-cedar-policy-engine-with-toml-front-end.md).
  Every `ADR-NNNN` mention must refer to an existing ADR. **(checked)**
- Every statement about a third-party product (a host's hook format, a competitor's features)
  links its source and gives the date it was verified: "(verified 2026-09-14)".

## One home per fact

Each fact below is defined in exactly one place.
Other pages link to that place instead of restating the fact, so the fact cannot drift out of sync.

| Fact | Home |
|---|---|
| Term definitions | [Glossary](../glossary.md) |
| Action types | [Action types](../spec/action-types.md) |
| Reason codes | [Decision](../spec/decision.md) |
| Risk signals | [Risk signals](../spec/risk-signals.md) |
| Failure semantics | [Security model](../security/security-model.md) |
| Enforcement level of each adapter | [Enforcement levels](../security/enforcement-levels.md) |
| Host hook formats | the host's page under `guides/integrations/` |
| Latency and memory budgets | [Performance budgets](../engineering/performance-budgets.md) |
| Platform tiers | [Platform support](../reference/platform-support.md) |
| Crates and their responsibilities | [Workspace and crates](../engineering/workspace-and-crates.md) |
| MSRV and toolchain | [Toolchain and MSRV](../engineering/toolchain-and-msrv.md) |
| Error codes | [Error codes](../reference/error-codes.md) |
| Milestones | [Roadmap](roadmap.md) |
| Decisions | [ADR index](../adr/README.md) |

## Placeholders

Information only a maintainer can supply, such as a contact address, is written as `TODO(maintainer): <what is needed>`.
Placeholders are allowed during the specification phase and must be resolved before the first release.

## Checklist for documentation changes

- [ ] Metadata and banner are correct, and `reviewed` is updated.
- [ ] The page is listed in `SUMMARY.md`.
- [ ] New terms are added to the glossary, and existing terms are used as defined.
- [ ] Each fact is stated in its home, and other pages link to it.
- [ ] JSON examples carry validation markers.
- [ ] Claims about third-party products link a source and a verification date.
- [ ] `cargo xtask docs-check`, `mdbook build docs`, `lychee`, `typos`, and `rumdl` pass locally.
