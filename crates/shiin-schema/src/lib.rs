//! Wire types for Shiin's specifications.
//!
//! Every type here mirrors a JSON Schema under `docs/spec/schemas/`.
//! Action requests accept unknown fields, because a reader ignores unknown
//! non-critical fields. Policy and task declarations reject them, because an
//! unrecognized policy key must fail closed rather than be silently dropped.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A decision value, as defined in `docs/spec/decision.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecisionValue {
    /// The action may proceed.
    Allow,
    /// The action must not proceed.
    Deny,
    /// The action waits for a human.
    Pending,
}

/// How certain Shiin is about an effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Verified by Shiin, such as a canonicalized path.
    Exact,
    /// Believed but unverified.
    Inferred,
    /// Not determinable before the action runs.
    Opaque,
}

/// How strongly an agent's identity is established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Assurance {
    /// The adapter read the identity from the host payload.
    Asserted,
    /// Confirmed by operating system peer credentials.
    AttestedLocal,
    /// The agent runs as a dedicated operating system user.
    AttestedOsUser,
    /// Reserved for future cryptographic attestation.
    AttestedCrypto,
}

/// Whether an approver can be reached, per `EVAL-016`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// A human can resolve a pending decision.
    #[default]
    Attended,
    /// No human is available, so a pending decision becomes a denial.
    Unattended,
}

/// The agent that proposed an action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Agent {
    /// Host-defined agent identifier, such as `claude-code`.
    pub id: String,
    /// How strongly the identifier is established.
    pub assurance: Assurance,
}

/// The project an agent is working in.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Workspace {
    /// Absolute, canonical path of the workspace root.
    pub root: String,
    /// Version control system in use, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcs: Option<String>,
}

/// The host that proposed the action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Host {
    /// Host name, such as `claude-code`.
    pub name: String,
    /// The host's own name for the tool, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

/// One concrete consequence of an action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Effect {
    /// What the value refers to: `path`, `command`, `url`, or `count`.
    pub kind: String,
    /// The value itself, canonicalized when it is a path.
    pub value: String,
    /// How certain Shiin is about this effect.
    pub confidence: Confidence,
}

impl Effect {
    /// Whether this effect is of the given kind.
    #[must_use]
    pub fn is(&self, kind: &str) -> bool {
        self.kind == kind
    }

    /// The effect's value parsed as a count, when it is one.
    #[must_use]
    pub fn count(&self) -> Option<u64> {
        if self.is("count") {
            self.value.parse().ok()
        } else {
            None
        }
    }
}

/// A normalized, host-independent description of one proposed action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionRequest {
    /// Specification version of this message.
    pub schema: String,
    /// Unique identifier of the request.
    pub id: String,
    /// The host session the action belongs to.
    pub session: String,
    /// The agent that proposed the action.
    pub agent: Agent,
    /// The normalized action type, such as `fs.write`.
    pub action_type: String,
    /// The action's effects.
    #[serde(default)]
    pub effects: Vec<Effect>,
    /// Identifier of the active task, if any.
    #[serde(default)]
    pub task: Option<String>,
    /// The workspace the action takes place in.
    pub workspace: Workspace,
    /// Digest of the policy snapshot the decision must use.
    pub policy_snapshot: String,
    /// The host that proposed the action.
    pub host: Host,
}

impl ActionRequest {
    /// Values of every effect of the given kind.
    pub fn values_of(&self, kind: &str) -> impl Iterator<Item = &str> {
        self.effects
            .iter()
            .filter(move |effect| effect.is(kind))
            .map(|effect| effect.value.as_str())
    }

    /// Whether any effect is opaque.
    #[must_use]
    pub fn has_opaque_effect(&self) -> bool {
        self.effects
            .iter()
            .any(|effect| effect.confidence == Confidence::Opaque)
    }
}

/// One policy rule.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Unique identifier, which appears in decisions and traces.
    pub id: String,
    /// What the rule produces when it matches.
    pub effect: DecisionValue,
    /// Action type patterns the rule applies to. A trailing `*` is a wildcard.
    #[serde(default)]
    pub action_types: Vec<String>,
    /// Path globs the rule applies to.
    #[serde(default)]
    pub resources: Vec<String>,
    /// Command globs the rule applies to.
    #[serde(default)]
    pub commands: Vec<String>,
    /// Agent identifiers the rule applies to. Empty means every agent.
    #[serde(default)]
    pub agents: Vec<String>,
    /// Operator-facing explanation, shown when the rule decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One source of policy.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    /// Layer name: `built-in`, `user`, or `workspace`.
    pub name: String,
    /// Digest of the layer's source.
    pub digest: String,
    /// The layer's rules.
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// Configuration for one risk signal.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RiskSetting {
    /// Shorthand for an escalation with default settings.
    Escalation(DecisionValue),
    /// An escalation with explicit settings.
    Detailed {
        /// What the signal escalates to.
        decision: DecisionValue,
        /// Threshold for signals that count, such as `bulk_delete`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        threshold: Option<u64>,
    },
}

impl RiskSetting {
    /// What this signal escalates to.
    #[must_use]
    pub fn decision(&self) -> DecisionValue {
        match *self {
            Self::Escalation(decision) | Self::Detailed { decision, .. } => decision,
        }
    }

    /// The configured threshold, if any.
    #[must_use]
    pub fn threshold(&self) -> Option<u64> {
        match *self {
            Self::Escalation(_) => None,
            Self::Detailed { threshold, .. } => threshold,
        }
    }
}

/// A complete policy snapshot's source form.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    /// Specification version of this policy.
    pub schema: String,
    /// Policy layers, in precedence order.
    pub layers: Vec<Layer>,
    /// The decision when no rule matches. Never `allow`.
    pub default: DecisionValue,
    /// Risk signal configuration, keyed by signal name.
    #[serde(default)]
    pub risk: BTreeMap<String, RiskSetting>,
}

impl Policy {
    /// Every rule across every layer, in precedence order.
    pub fn rules(&self) -> impl Iterator<Item = &Rule> {
        self.layers.iter().flat_map(|layer| layer.rules.iter())
    }
}

/// Paths a task may touch.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    /// Path globs the agent may write within.
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Action types a task may not perform.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Forbid {
    /// Action type patterns that are refused.
    #[serde(default)]
    pub action_types: Vec<String>,
}

/// A human-confirmed statement of what an agent is working on.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskDeclaration {
    /// Identifier referenced by an action request.
    pub id: String,
    /// What the agent was asked to do, recorded but never evaluated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Paths the task is limited to.
    #[serde(default)]
    pub scope: Scope,
    /// Action types the task forbids.
    #[serde(default)]
    pub forbid: Forbid,
    /// Action types that always need approval within this task.
    #[serde(default)]
    pub require_pending_for: Vec<String>,
    /// When the task ends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// The engine's answer to an action request.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Decision {
    /// Specification version of this message.
    pub schema: String,
    /// The request this answers.
    pub request_id: String,
    /// The outcome.
    pub decision: DecisionValue,
    /// Stable reason code, such as `deny.policy`.
    pub reason: String,
    /// The stage that determined the outcome.
    pub stage: String,
    /// Identifiers of the rules that determined the outcome.
    #[serde(default)]
    pub matched_rules: Vec<String>,
    /// Digest of the policy snapshot used.
    pub policy_snapshot: String,
    /// Version of the engine that decided.
    pub engine_version: String,
}

/// Wire version strings this release reads and writes.
pub mod versions {
    /// Action request version.
    pub const ACTION_REQUEST: &str = "shiin.action-request/v1-draft.1";
    /// Decision version.
    pub const DECISION: &str = "shiin.decision/v1-draft.1";
    /// Policy version.
    pub const POLICY: &str = "shiin.policy/v1-draft.1";
}

/// Reason codes, as registered in `docs/spec/decision.md`.
pub mod reasons {
    /// A policy rule allowed the action.
    pub const ALLOW_POLICY: &str = "allow.policy";
    /// A policy rule denied the action.
    pub const DENY_POLICY: &str = "deny.policy";
    /// The action does not fit the active task.
    pub const DENY_INTENT: &str = "deny.intent";
    /// A risk signal denied the action.
    pub const DENY_RISK: &str = "deny.risk";
    /// No rule matched, so the default decision applied.
    pub const DENY_DEFAULT: &str = "deny.default";
    /// A failure produced a fail-closed denial.
    pub const DENY_FAILURE: &str = "deny.failure";
    /// No approver was available to resolve a pending decision.
    pub const DENY_UNATTENDED: &str = "deny.unattended";
    /// The action needs a grant or an approval.
    pub const PENDING_GRANTABLE: &str = "pending.grantable";
    /// The action needs confirmation against the task.
    pub const PENDING_INTENT: &str = "pending.intent";
    /// A risk signal escalated the action.
    pub const PENDING_RISK: &str = "pending.risk";
    /// An opaque effect needs approval.
    pub const PENDING_OPAQUE: &str = "pending.opaque";
}
