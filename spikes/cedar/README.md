# ADR-0014 spike: Cedar as the policy engine

This directory holds the spike that [ADR-0014](../../docs/adr/0014-cedar-policy-engine-with-toml-front-end.md)
required before the decision could be accepted.
It is throwaway code, not product code, and nothing in it is normative.

```console
$ cargo run -p spike-cedar
```

## Question

Can Cedar express Shiin's decision semantics, specifically three outcomes
(`allow`, `deny`, `pending`), deny-overrides combining, path and command globs,
and reason codes that say which stage decided?

## Method

The spike reads every test vector in `docs/spec/test-vectors/`, compiles that
vector's policy into Cedar, evaluates it, applies the combining rules from
[the evaluation specification](../../docs/spec/evaluation.md), and compares the
result with the vector's expectation.

Each policy rule becomes one Cedar policy whose identifier is the rule's identifier:

- a `deny` rule becomes `forbid`,
- an `allow` rule becomes `permit`,
- a `pending` rule becomes `permit` carrying `@decision("pending")`.

Glob matching runs in Rust with `globset` and reaches Cedar as a context attribute,
`context.matched_rules`, which holds the identifiers of the rules whose matchers the
request satisfies.

## Result: Cedar is suitable

All 8 vectors reproduce their expected decision and reason code.

| Vector | Case | Outcome |
|---|---|---|
| TV-001 | An allowed edit | `allow` / `allow.policy` |
| TV-002 | A denied read of a secret | `deny` / `deny.policy` |
| TV-003 | A command with unknown effects | `pending` / `pending.opaque` |
| TV-004 | An action outside the task | `deny` / `deny.intent` |
| TV-005 | A bulk deletion | `pending` / `pending.risk` |
| TV-006 | A destructive version-control operation | `pending` / `pending.risk` |
| TV-007 | An unattended deny | `deny` / `deny.unattended` |
| TV-008 | No rule matches | `deny` / `deny.default` |

### What Cedar gives us directly

- **Deny overrides permit**, which is exactly "deny always wins". A matching `forbid`
  beats a matching `permit` with no extra logic, and the response's diagnostics name
  the policies that determined the answer, which is how the reason code and the
  matched rule identifiers are produced.
- **Explainability.** `Response::diagnostics().reason()` returns the determining
  policy identifiers, so an operator can be told which rule decided.
- **Stable identifiers.** A Cedar policy can be parsed with an explicit identifier,
  so a rule's identifier survives into the decision unchanged.

### What Cedar cannot do, and how the spike works around it

- **There is no third outcome.** Cedar answers `Allow` or `Deny` only. `pending` is
  carried as an annotation on a `permit`, and the caller downgrades an allow to
  pending when any determining policy is annotated. This works, and it means
  `pending` is Shiin's concept rather than Cedar's: the annotation is the contract.
- **`like` has a single wildcard** and no path semantics, so `**` cannot be expressed.
  Globs are therefore precomputed in Rust and passed in as context, as ADR-0014
  anticipated. The consequence is that Cedar never sees a path: it sees which rules
  matched. Policy analysis over paths is therefore outside Cedar's reach.
- **Gates are not policy.** The identity, capability, intent, and risk stages stay in
  Rust, because they are not authorization rules.

## Gaps this spike found in the specifications

Each of these was fixed before the spike was declared finished, and the spike then
re-run against the corrected specifications.

1. **No workspace root.** The action request had no workspace, so a workspace-relative
   glob such as `./src/**` could not be resolved and would have had to match anywhere,
   which is weaker than intended. Fixed by requiring `workspace.root` (AR-008, AR-009).
2. **No risk threshold.** `bulk_delete` is specified as using a configured threshold,
   but the policy schema had nowhere to put one. Fixed by allowing a risk entry to be
   an object with a `threshold`, with a bare string kept as shorthand (RISK-007a).
3. **No reason code for the default decision.** Nothing covered "no rule matched at
   all". Fixed by adding `deny.default`, and TV-008 now exercises it.
4. **Two undecidable vectors.** TV-004 expected `deny.intent` with no task constraints
   anywhere, and TV-007 expected an unattended denial with no way to say the engine was
   unattended. Fixed by adding the optional `task` and `mode` fields to the vector
   schema, and by defining the evaluation inputs and unattended mode in the evaluation
   specification (EVAL-014 to EVAL-017).

## Limits of this spike

- It implements only the policy, intent, and risk stages. Identity, capability, grants,
  and approvals are untouched.
- It reads the policy in its JSON form. The TOML front-end and its compilation are not
  exercised, so the ergonomics of writing policy remain unproven.
- It measures nothing. The latency budgets in
  [performance budgets](../../docs/engineering/performance-budgets.md) still need a
  benchmark, including Cedar's parse and evaluation cost at 1,000 rules.
- Path canonicalization is still open ([ADR-0019](../../docs/adr/0019-path-canonicalization.md)).
  The spike compares the strings the vectors supply.

## Disposition

Keep this directory until `shiin-policy` exists and its tests cover the same vectors,
then delete it. The conformance corpus, not this spike, is the lasting artifact.
