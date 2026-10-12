//! The decision engine: five stages, fixed order, no input beyond what is passed in.
//!
//! The engine is pure and synchronous, performs no I/O, and reads no clock
//! ([ADR-0010](../../../docs/adr/0010-pure-synchronous-engine.md)). The same inputs
//! always produce the same decision, which is what makes replay possible.
//!
//! Stage order and combining follow `docs/spec/evaluation.md`:
//! identity, capability, policy, intent, and risk. Identity, capability, intent, and
//! risk are gates: they may deny or hold an action, never allow it.

use shiin_policy::{CompiledPolicy, PolicyOutcome};
use shiin_schema::{
    ActionRequest, Decision, DecisionValue, Mode, Policy, TaskDeclaration, reasons, versions,
};

/// Version reported in decisions.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default threshold for `bulk_delete`, used when policy configures none (`RISK-007a`).
pub const DEFAULT_BULK_DELETE_THRESHOLD: u64 = 20;

/// What one stage concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Stage {
    /// Refuse the action, with a reason code.
    Deny(&'static str, Vec<String>),
    /// Hold the action for a human, with a reason code.
    Pending(&'static str, Vec<String>),
    /// Permit the action. Only the policy stage may return this.
    Allow(Vec<String>),
    /// This stage has nothing to say.
    Pass,
}

/// Everything the engine is allowed to read, per `EVAL-014`.
pub struct Inputs<'a> {
    /// The action being decided.
    pub request: &'a ActionRequest,
    /// The compiled policy snapshot.
    pub policy: &'a CompiledPolicy,
    /// The source policy, read for risk configuration.
    pub source: &'a Policy,
    /// The active task declaration, if any.
    pub task: Option<&'a TaskDeclaration>,
    /// Whether an approver can be reached.
    pub mode: Mode,
}

/// Decides one action request.
///
/// A failure inside evaluation produces `deny` with reason `deny.failure`
/// (`EVAL-009`), so this function cannot fail.
#[must_use]
pub fn decide(inputs: &Inputs<'_>) -> Decision {
    let identity = identity_stage(inputs.request);
    let capability = capability_stage();
    let policy = match inputs.policy.evaluate(inputs.request) {
        Ok(outcome) => policy_stage(&outcome),
        // Fail closed: an engine error is never an allow.
        Err(_) => Stage::Deny(reasons::DENY_FAILURE, Vec::new()),
    };
    let intent = intent_stage(inputs.request, inputs.task);
    let risk = risk_stage(inputs.request, inputs.source);

    let stages = [&identity, &capability, &policy, &intent, &risk];
    let (decision, reason, stage_name, rules) = combine(&stages, &policy, inputs);

    Decision {
        schema: versions::DECISION.to_owned(),
        request_id: inputs.request.id.clone(),
        decision,
        reason: reason.to_owned(),
        stage: stage_name.to_owned(),
        matched_rules: rules,
        policy_snapshot: inputs.request.policy_snapshot.clone(),
        engine_version: ENGINE_VERSION.to_owned(),
    }
}

/// Applies the combining rules in the order `EVAL-005` to `EVAL-008` fix.
fn combine(
    stages: &[&Stage; 5],
    policy: &Stage,
    inputs: &Inputs<'_>,
) -> (DecisionValue, &'static str, &'static str, Vec<String>) {
    const NAMES: [&str; 5] = ["identity", "capability", "policy", "intent", "risk"];

    for (stage, name) in stages.iter().zip(NAMES) {
        if let Stage::Deny(reason, rules) = stage {
            return (DecisionValue::Deny, reason, name, rules.clone());
        }
    }
    for (stage, name) in stages.iter().zip(NAMES) {
        if let Stage::Pending(reason, rules) = stage {
            // An unattended engine cannot hold an action for a human (`EVAL-017`).
            return if inputs.mode == Mode::Unattended {
                (
                    DecisionValue::Deny,
                    reasons::DENY_UNATTENDED,
                    name,
                    rules.clone(),
                )
            } else {
                (DecisionValue::Pending, reason, name, rules.clone())
            };
        }
    }
    if let Stage::Allow(rules) = policy {
        return (
            DecisionValue::Allow,
            reasons::ALLOW_POLICY,
            "policy",
            rules.clone(),
        );
    }
    // Nothing matched, so the default decision applies. It is never allow.
    let default = inputs.policy.default_decision();
    if default == DecisionValue::Pending && inputs.mode == Mode::Attended {
        (
            DecisionValue::Pending,
            reasons::PENDING_GRANTABLE,
            "default",
            Vec::new(),
        )
    } else if default == DecisionValue::Pending {
        (
            DecisionValue::Deny,
            reasons::DENY_UNATTENDED,
            "default",
            Vec::new(),
        )
    } else {
        (
            DecisionValue::Deny,
            reasons::DENY_DEFAULT,
            "default",
            Vec::new(),
        )
    }
}

/// Identity gate. v0.1 accepts any asserted identity and records its weakness.
fn identity_stage(_request: &ActionRequest) -> Stage {
    Stage::Pass
}

/// Capability gate. The v0.1 policy model has no capability element yet.
fn capability_stage() -> Stage {
    Stage::Pass
}

/// Maps the policy stage's outcome onto a stage result.
fn policy_stage(outcome: &PolicyOutcome) -> Stage {
    match outcome {
        PolicyOutcome::Allow(rules) => Stage::Allow(rules.clone()),
        PolicyOutcome::Deny(rules) => Stage::Deny(reasons::DENY_POLICY, rules.clone()),
        PolicyOutcome::Pending(rules) => Stage::Pending(reasons::PENDING_GRANTABLE, rules.clone()),
        PolicyOutcome::NoMatch => Stage::Pass,
    }
}

/// Intent gate: checks the action against the active task's constraints.
fn intent_stage(request: &ActionRequest, task: Option<&TaskDeclaration>) -> Stage {
    let Some(task) = task else {
        return Stage::Pass;
    };
    if task
        .forbid
        .action_types
        .iter()
        .any(|pattern| pattern_matches(pattern, &request.action_type))
    {
        return Stage::Deny(reasons::DENY_INTENT, vec![task.id.clone()]);
    }
    if task
        .require_pending_for
        .iter()
        .any(|pattern| pattern_matches(pattern, &request.action_type))
    {
        return Stage::Pending(reasons::PENDING_INTENT, vec![task.id.clone()]);
    }
    if !task.scope.paths.is_empty() {
        let mut touched = request.values_of("path").peekable();
        if touched.peek().is_some()
            && touched.any(|path| {
                !task
                    .scope
                    .paths
                    .iter()
                    .any(|pattern| glob_matches(pattern, path))
            })
        {
            return Stage::Deny(reasons::DENY_INTENT, vec![task.id.clone()]);
        }
    }
    Stage::Pass
}

/// Risk gate: raises the signals the policy configures, and escalates accordingly.
fn risk_stage(request: &ActionRequest, policy: &Policy) -> Stage {
    let mut raised: Vec<(&'static str, DecisionValue)> = Vec::new();
    let mut consider = |signal: &str, code: &'static str, fired: bool| {
        if fired && let Some(setting) = policy.risk.get(signal) {
            raised.push((code, setting.decision()));
        }
    };

    consider(
        "opaque_command",
        reasons::PENDING_OPAQUE,
        request.has_opaque_effect(),
    );
    let threshold = policy
        .risk
        .get("bulk_delete")
        .and_then(shiin_schema::RiskSetting::threshold)
        .unwrap_or(DEFAULT_BULK_DELETE_THRESHOLD);
    let deleted = request
        .effects
        .iter()
        .filter_map(shiin_schema::Effect::count)
        .max()
        .unwrap_or(0);
    consider("bulk_delete", reasons::PENDING_RISK, deleted > threshold);
    consider(
        "destructive_vcs",
        reasons::PENDING_RISK,
        request.values_of("command").any(is_destructive_vcs),
    );
    consider(
        "privilege_escalation",
        reasons::PENDING_RISK,
        request.values_of("command").any(is_privilege_escalation),
    );

    if raised
        .iter()
        .any(|(_, decision)| *decision == DecisionValue::Deny)
    {
        return Stage::Deny(reasons::DENY_RISK, Vec::new());
    }
    if let Some((code, _)) = raised
        .iter()
        .find(|(_, decision)| *decision == DecisionValue::Pending)
    {
        return Stage::Pending(code, Vec::new());
    }
    Stage::Pass
}

/// Whether a command rewrites or discards version control history.
fn is_destructive_vcs(command: &str) -> bool {
    command.starts_with("git")
        && ["--force", "-f ", "reset --hard", "branch -D", "clean -fd"]
            .iter()
            .any(|marker| command.contains(marker))
}

/// Whether a command asks for higher privileges.
fn is_privilege_escalation(command: &str) -> bool {
    ["sudo ", "doas ", "su ", "pkexec ", "runas "]
        .iter()
        .any(|prefix| command.starts_with(prefix))
}

/// Matches an action type pattern, where a trailing `*` is a wildcard.
fn pattern_matches(pattern: &str, action_type: &str) -> bool {
    pattern
        .strip_suffix('*')
        .map_or(pattern == action_type, |prefix| {
            action_type.starts_with(prefix)
        })
}

/// Matches a path glob, treating a leading `./` as workspace-relative.
fn glob_matches(pattern: &str, value: &str) -> bool {
    let rewritten = pattern
        .strip_prefix("./")
        .map_or_else(|| pattern.to_owned(), |rest| format!("**/{rest}"));
    globset::Glob::new(&rewritten).is_ok_and(|glob| glob.compile_matcher().is_match(value))
}
