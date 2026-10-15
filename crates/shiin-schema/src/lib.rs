//! Wire types for Shiin's specifications.
//!
//! Every type here mirrors a JSON Schema under `docs/spec/schemas/`.
//! Top-level action requests, policy, and task declarations reject unknown fields,
//! because an unrecognized constraint must not be silently dropped.

pub mod clock;

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
#[serde(deny_unknown_fields)]
pub struct Agent {
    /// Host-defined agent identifier, such as `claude-code`.
    pub id: String,
    /// How strongly the identifier is established.
    pub assurance: Assurance,
}

/// The project an agent is working in.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Effect {
    /// What the value refers to: `path`, `command`, `url`, or `count`.
    pub kind: String,
    /// The value itself, canonicalized when it is a path.
    pub value: String,
    /// How certain Shiin is about this effect.
    pub confidence: Confidence,
    /// The action type this effect represents, when it differs from the request's own.
    ///
    /// A shell command that reads a file carries `fs.read` here, so a rule written
    /// for `fs.read` covers the read however the agent performed it (`AR-010`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
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
#[serde(deny_unknown_fields)]
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
    /// Checks the required wire fields before evaluation (`AR-001` to `AR-008`).
    ///
    /// # Errors
    ///
    /// Returns a fixed diagnostic for an unsupported version or malformed required field.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema != versions::ACTION_REQUEST {
            return Err("unsupported action request schema");
        }
        let id = self.id.as_bytes();
        let uuid = id.len() == 36
            && id.iter().enumerate().all(|(index, byte)| {
                if [8, 13, 18, 23].contains(&index) {
                    *byte == b'-'
                } else {
                    byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()
                }
            })
            && id.get(14) == Some(&b'4')
            && id.get(19).is_some_and(|byte| b"89ab".contains(byte));
        if !uuid {
            return Err("action request id must be a version 4 UUID");
        }
        if self.session.is_empty()
            || self.agent.id.is_empty()
            || self.action_type.is_empty()
            || self.host.name.is_empty()
            || self.effects.is_empty()
            || self.effects.iter().any(|effect| effect.kind.is_empty())
        {
            return Err("action request is missing a required value");
        }
        let root = self.workspace.root.as_bytes();
        let windows_absolute = root.first().is_some_and(u8::is_ascii_alphabetic)
            && root.get(1) == Some(&b':')
            && root.get(2).is_some_and(|byte| b"/\\".contains(byte));
        if !self.workspace.root.starts_with('/')
            && !windows_absolute
            && !self.workspace.root.starts_with("\\\\")
        {
            return Err("workspace root must be absolute");
        }
        if !self
            .policy_snapshot
            .strip_prefix("sha256:")
            .is_some_and(|digest| {
                digest.len() == 64
                    && digest
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            })
        {
            return Err("policy snapshot must be a SHA-256 digest");
        }
        Ok(())
    }

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
#[serde(untagged, deny_unknown_fields)]
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

/// Hosts a task may reach.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllowHosts {
    /// Hosts the agent may contact. Empty means the constraint is not set.
    #[serde(default)]
    pub hosts: Vec<String>,
}

/// Counting limits a task may not exceed.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Most files the task may write or create.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_written: Option<u64>,
    /// Most files the task may delete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_deleted: Option<u64>,
    /// Most commands the task may run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<u64>,
}

/// What a task has used so far.
///
/// Counters are an input to evaluation rather than state inside the engine, so that
/// replaying a recorded request reproduces its decision exactly.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct TaskUsage {
    /// Files written or created so far.
    pub files_written: u64,
    /// Files deleted so far.
    pub files_deleted: u64,
    /// Commands run so far.
    pub commands: u64,
}

impl TaskUsage {
    /// Adds another action's usage to this one.
    #[must_use]
    pub fn plus(self, other: Self) -> Self {
        Self {
            files_written: self.files_written.saturating_add(other.files_written),
            files_deleted: self.files_deleted.saturating_add(other.files_deleted),
            commands: self.commands.saturating_add(other.commands),
        }
    }
}

/// A human-confirmed statement of what an agent is working on.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    /// Hosts the task may reach.
    #[serde(default)]
    pub allow: AllowHosts,
    /// Counting limits the task may not exceed.
    #[serde(default)]
    pub limits: Limits,
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
    /// Fixed agent-facing explanation, without policy internals (`DEC-009`).
    pub message: String,
    /// The approval request for a held action, when a daemon is configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<ApprovalRequest>,
    /// Results of all five stages, available to the operator in explain mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<StageTrace>>,
    /// The stage that determined the outcome.
    #[serde(skip)]
    pub stage: String,
    /// Identifiers of the rules that determined the outcome.
    #[serde(skip)]
    pub matched_rules: Vec<String>,
    /// Digest of the policy snapshot used.
    #[serde(skip)]
    pub policy_snapshot: String,
    /// Version of the engine that decided.
    #[serde(skip)]
    pub engine_version: String,
}

/// A human's resolution of a pending action (`APR-004`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    /// Authorize one retry of exactly the normalized action.
    AllowOnce,
    /// Authorize a bounded, explicitly selected scope.
    AllowAlways,
    /// Refuse the pending action.
    Deny,
}

/// The versioned approval request, separate from the normalized action shown to a human.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalRequest {
    /// Wire version.
    pub schema: String,
    /// Original request identifier.
    pub request_id: String,
    /// Always pending for this request type.
    pub decision: DecisionValue,
    /// Stable reason for the hold.
    pub reason: String,
    /// Agent-facing generic explanation.
    pub message: String,
    /// Unpredictable identifier containing the normalized action binding digest.
    pub ticket: String,
    /// UTC creation time.
    pub created_at: String,
    /// UTC deadline, after which the request cannot be resolved.
    pub expires_at: String,
    /// Human resolution, if one exists.
    pub resolution: Option<Resolution>,
}

/// One stage's result in an operator-facing decision trace (`EVAL-011`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StageTrace {
    /// Stage name, in the fixed evaluation order.
    pub stage: String,
    /// `allow`, `deny`, `pending`, or `pass`.
    pub result: String,
    /// Reason code or rule identifiers that explain this result.
    pub reason: String,
}

impl DecisionValue {
    /// Agent-facing text that does not disclose the deciding rule or stage.
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Self::Allow => "This action is allowed.",
            Self::Deny => "This action is denied by Shiin.",
            Self::Pending => "This action needs a person's approval before it can proceed.",
        }
    }
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
