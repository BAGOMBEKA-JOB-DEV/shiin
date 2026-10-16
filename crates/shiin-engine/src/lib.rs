//! The decision engine: five stages, fixed order, no input beyond what is passed in.
//!
//! The engine is pure and synchronous, performs no I/O, and reads no clock
//! ([ADR-0010](../../../docs/adr/0010-pure-synchronous-engine.md)). The same inputs
//! always produce the same decision, which is what makes replay possible.
//!
//! Stage order and combining follow `docs/spec/evaluation.md`:
//! identity, capability, policy, intent, and risk. Identity, capability, intent, and
//! risk are gates: they may deny or hold an action, never allow it.

pub mod vectors;

use shiin_policy::{CompiledPolicy, PolicyOutcome};
use shiin_schema::{
    ActionRequest, Decision, DecisionValue, Mode, Policy, StageTrace, TaskDeclaration, TaskUsage,
    reasons, versions,
};

/// Version reported in decisions.
pub const ENGINE_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+capabilities.1");

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
    /// What the active task has used so far.
    pub usage: TaskUsage,
    /// The current time, as an RFC 3339 timestamp. The engine never reads a clock.
    pub now: &'a str,
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
    let capability = capability_stage(inputs.request, inputs.source);
    let policy = match inputs.policy.evaluate(inputs.request) {
        Ok(outcome) => policy_stage(&outcome),
        // Fail closed: an engine error is never an allow.
        Err(_) => Stage::Deny(reasons::DENY_FAILURE, Vec::new()),
    };
    let intent = intent_stage(inputs);
    let risk = risk_stage(inputs.request, inputs.source);

    let stages = [&identity, &capability, &policy, &intent, &risk];
    let (decision, reason, stage_name, rules) = combine(&stages, &policy, inputs);
    let trace = stages
        .iter()
        .zip(["identity", "capability", "policy", "intent", "risk"])
        .map(|(stage, name)| {
            let (result, reason) = match stage {
                Stage::Deny(reason, _) => ("deny", (*reason).to_owned()),
                Stage::Pending(reason, _) => ("pending", (*reason).to_owned()),
                Stage::Allow(rules) => ("allow", format!("rule:{}", rules.join(","))),
                Stage::Pass => ("pass", format!("{name}.pass")),
            };
            StageTrace {
                stage: name.to_owned(),
                result: result.to_owned(),
                reason,
            }
        })
        .collect();

    Decision {
        schema: versions::DECISION.to_owned(),
        request_id: inputs.request.id.clone(),
        decision,
        reason: reason.to_owned(),
        message: decision.message().to_owned(),
        approval: None,
        trace: Some(trace),
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

/// Every action/effect class needs an explicit grant; any matching refusal wins.
fn capability_stage(request: &ActionRequest, policy: &Policy) -> Stage {
    if policy
        .capabilities
        .iter()
        .any(|(key, capability)| capability.validate(key).is_err())
    {
        return Stage::Deny(reasons::DENY_FAILURE, Vec::new());
    }
    for action_type in std::iter::once(request.action_type.as_str()).chain(
        request
            .effects
            .iter()
            .filter_map(|effect| effect.action_type.as_deref()),
    ) {
        let mut granted = false;
        for capability in policy.capabilities.values().filter(|capability| {
            (capability.agents.is_empty() || capability.agents.contains(&request.agent.id))
                && capability.action_types.iter().any(|pattern| {
                    pattern
                        .strip_suffix('*')
                        .map_or(pattern == action_type, |prefix| {
                            action_type.starts_with(prefix)
                        })
                })
        }) {
            if capability.deny == Some(true) {
                return Stage::Deny(reasons::DENY_CAPABILITY, vec![capability.id.clone()]);
            }
            granted |= capability.allow == Some(true);
        }
        if !granted {
            return Stage::Deny(reasons::DENY_CAPABILITY, Vec::new());
        }
    }
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
///
/// Every constraint can only refuse or hold an action. A task never permits something
/// policy refuses ([ADR-0020](../../../docs/adr/0020-structured-intent-binding.md)).
fn intent_stage(inputs: &Inputs<'_>) -> Stage {
    let Some(task) = inputs.task else {
        return Stage::Pass;
    };
    // An expired task has ended, so its constraints no longer apply. Policy still does.
    if task
        .expires_at
        .as_deref()
        .is_some_and(|expiry| inputs.now > expiry)
    {
        return Stage::Pass;
    }
    let request = inputs.request;
    let reference = vec![task.id.clone()];

    let matches_action = |pattern: &str| {
        std::iter::once(request.action_type.as_str())
            .chain(
                request
                    .effects
                    .iter()
                    .filter_map(|effect| effect.action_type.as_deref()),
            )
            .any(|action_type| pattern_matches(pattern, action_type))
    };
    if task
        .forbid
        .action_types
        .iter()
        .any(|pattern| matches_action(pattern))
    {
        return Stage::Deny(reasons::DENY_INTENT, reference);
    }
    if !task.scope.paths.is_empty() && touches_outside_scope(request, task) {
        return Stage::Deny(reasons::DENY_INTENT, reference);
    }
    if !task.allow.hosts.is_empty() && reaches_other_host(request, task) {
        return Stage::Deny(reasons::DENY_INTENT, reference);
    }
    if exceeds_limits(inputs, task) {
        return Stage::Deny(reasons::DENY_INTENT, reference);
    }
    if task
        .require_pending_for
        .iter()
        .any(|pattern| matches_action(pattern))
    {
        return Stage::Pending(reasons::PENDING_INTENT, reference);
    }
    Stage::Pass
}

/// Whether the action writes, creates, deletes, or moves a path outside the task's scope.
///
/// Reads are not scope-limited: a task that forbade reading its own repository would
/// stop an agent from working at all, and policy already governs what may be read.
fn touches_outside_scope(request: &ActionRequest, task: &TaskDeclaration) -> bool {
    const CHANGING: [&str; 4] = ["fs.write", "fs.create", "fs.delete", "fs.move"];

    let changes = |action_type: &str| CHANGING.contains(&action_type);
    let mut outside = false;
    for effect in &request.effects {
        if !effect.is("path") {
            continue;
        }
        let action_type = effect
            .action_type
            .as_deref()
            .unwrap_or(&request.action_type);
        if !changes(action_type) {
            continue;
        }
        if !task.scope.paths.iter().any(|pattern| {
            let resolved = pattern.strip_prefix("./").map_or_else(
                || pattern.to_owned(),
                |rest| format!("{}/{rest}", request.workspace.root.trim_end_matches('/')),
            );
            glob_matches(&resolved, &effect.value)
        }) {
            outside = true;
        }
    }
    outside
}

/// Whether the action reaches a host the task did not allow.
fn reaches_other_host(request: &ActionRequest, task: &TaskDeclaration) -> bool {
    request.values_of("url").any(|value| {
        let host = host_of(value);
        !task
            .allow
            .hosts
            .iter()
            .any(|allowed| host_matches(allowed, &host))
    })
}

/// The host part of a URL, or the value itself when it is not a URL.
fn host_of(value: &str) -> String {
    let without_scheme = value.split_once("://").map_or(value, |(_, rest)| rest);
    without_scheme
        .split(['/', ':', '?'])
        .next()
        .unwrap_or(without_scheme)
        .to_lowercase()
}

/// Whether a host pattern matches a host, where a leading `*.` is a label wildcard.
fn host_matches(pattern: &str, host: &str) -> bool {
    pattern.strip_prefix("*.").map_or_else(
        || pattern.eq_ignore_ascii_case(host),
        |suffix| host == suffix || host.ends_with(&format!(".{suffix}")),
    )
}

/// Whether performing this action would take the task past one of its limits.
fn exceeds_limits(inputs: &Inputs<'_>, task: &TaskDeclaration) -> bool {
    let after = inputs.usage.plus(usage_of(inputs.request));
    let past = |limit: Option<u64>, used: u64| limit.is_some_and(|limit| used > limit);

    past(task.limits.files_written, after.files_written)
        || past(task.limits.files_deleted, after.files_deleted)
        || past(task.limits.commands, after.commands)
}

/// What one action adds to a task's counters.
///
/// A caller applies this only after a decision of `allow`, so a refused action never
/// consumes a task's budget.
#[must_use]
pub fn usage_of(request: &ActionRequest) -> TaskUsage {
    let mut usage = TaskUsage::default();
    let kinds = std::iter::once(request.action_type.as_str()).chain(
        request
            .effects
            .iter()
            .filter_map(|effect| effect.action_type.as_deref()),
    );
    for action_type in kinds {
        match action_type {
            "fs.write" | "fs.create" => usage.files_written = usage.files_written.saturating_add(1),
            "fs.delete" => {
                // A deletion may report how many files it covers.
                let counted = request
                    .effects
                    .iter()
                    .filter_map(shiin_schema::Effect::count)
                    .max()
                    .unwrap_or(1);
                usage.files_deleted = usage.files_deleted.saturating_add(counted);
            }
            "shell.exec" => usage.commands = usage.commands.saturating_add(1),
            _ => {}
        }
    }
    usage
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
    consider(
        "secret_path",
        reasons::PENDING_RISK,
        request.values_of("path").any(|path| {
            ["**/.env", "**/.env.*", "**/.ssh/**", "**/credentials"]
                .iter()
                .any(|pattern| glob_matches(pattern, path))
        }),
    );
    consider(
        "outside_workspace",
        reasons::PENDING_RISK,
        request
            .values_of("path")
            .any(|path| !std::path::Path::new(path).starts_with(&request.workspace.root)),
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

#[cfg(test)]
mod tests {
    use super::{host_matches, host_of, usage_of};
    use shiin_schema::{ActionRequest, Agent, Assurance, Confidence, Effect, Host, Workspace};

    /// A workspace root that exists on no platform.
    ///
    /// Canonicalization resolves real symlinks, so a root that happens to exist is not a
    /// fixed point: macOS rewrites `/home` through autofs.
    const ROOT: &str = "/shiin-test-root/project";

    fn request(action_type: &str, effects: Vec<Effect>) -> ActionRequest {
        ActionRequest {
            schema: shiin_schema::versions::ACTION_REQUEST.to_owned(),
            id: "0190f3a0-1c2d-7e3f-8a4b-5c6d7e8f9a0b".to_owned(),
            session: "s".to_owned(),
            agent: Agent {
                id: "claude-code".to_owned(),
                assurance: Assurance::Asserted,
            },
            action_type: action_type.to_owned(),
            effects,
            task: None,
            workspace: Workspace {
                root: ROOT.to_owned(),
                vcs: None,
            },
            policy_snapshot: "sha256:0".to_owned(),
            host: Host {
                name: "claude-code".to_owned(),
                tool: None,
            },
        }
    }

    fn effect(kind: &str, value: &str, action_type: Option<&str>) -> Effect {
        Effect {
            kind: kind.to_owned(),
            value: value.to_owned(),
            confidence: Confidence::Exact,
            action_type: action_type.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn capability_selectors_restrict_and_malformed_snapshots_fail_closed() {
        let case: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/spec/test-vectors/tv-001-allowed-edit.v1.json"
        ))
        .expect("corpus fixture");
        let mut policy: shiin_schema::Policy =
            serde_json::from_value(case["policy"].clone()).expect("valid policy");
        let action: ActionRequest =
            serde_json::from_value(case["action_request"].clone()).expect("valid action");
        assert_eq!(
            super::capability_stage(&action, &policy),
            super::Stage::Pass
        );
        let capability = policy
            .capabilities
            .get_mut("EXAMPLE-corpus")
            .expect("explicit fixture grant");
        capability.agents = vec!["OTHER-EXAMPLE-agent".to_owned()];
        assert!(matches!(
            super::capability_stage(&action, &policy),
            super::Stage::Deny("deny.capability", _)
        ));
        let capability = policy
            .capabilities
            .get_mut("EXAMPLE-corpus")
            .expect("explicit fixture grant");
        capability.agents = vec![action.agent.id.clone()];
        capability.allow = Some(false);
        assert!(matches!(
            super::capability_stage(&action, &policy),
            super::Stage::Deny("deny.failure", _)
        ));
    }

    #[test]
    fn a_host_wildcard_matches_only_subdomains() {
        assert!(host_matches("docs.example.com", "docs.example.com"));
        assert!(host_matches("*.example.org", "api.example.org"));
        assert!(host_matches("*.example.org", "example.org"));
        assert!(!host_matches("*.example.org", "example.org.attacker.test"));
        assert!(!host_matches("docs.example.com", "paste.example.net"));
    }

    #[test]
    fn a_url_reduces_to_its_host() {
        assert_eq!(
            host_of("https://docs.example.com/guide?x=1"),
            "docs.example.com"
        );
        assert_eq!(
            host_of("http://paste.example.net:8080/a"),
            "paste.example.net"
        );
        assert_eq!(host_of("docs.example.com"), "docs.example.com");
    }

    #[test]
    fn a_shell_command_that_writes_counts_against_both_budgets() {
        let usage = usage_of(&request(
            "shell.exec",
            vec![
                effect("command", "cp a b", None),
                effect("path", "/shiin-test-root/project/b", Some("fs.write")),
            ],
        ));
        assert_eq!(usage.commands, 1);
        assert_eq!(usage.files_written, 1);
    }

    #[test]
    fn a_recursive_delete_counts_every_file_it_reports() {
        let mut deletion = request(
            "fs.delete",
            vec![
                effect("path", "/shiin-test-root/project/build", None),
                effect("count", "50", None),
            ],
        );
        deletion.effects[1].confidence = Confidence::Inferred;
        assert_eq!(usage_of(&deletion).files_deleted, 50);
    }
}
