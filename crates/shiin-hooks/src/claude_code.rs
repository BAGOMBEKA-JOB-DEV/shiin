//! The Claude Code hook adapter.
//!
//! Claude Code runs a program before each tool use and reads a decision back. This
//! module translates its `PreToolUse` payload into an action request, and a decision
//! into the response Claude Code expects
//! (verified against <https://code.claude.com/docs/en/hooks> on 2026-09-14).
//!
//! Enforcement level L1, cooperative: the host honors the decision, and the hook can
//! be disabled by the person running the host
//! ([enforcement levels](../../../docs/security/enforcement-levels.md)).
//!
//! The host treats a crashed, timed-out, or unparsable hook as an allow, so every
//! failure here must be turned into an explicit deny
//! ([ADR-0013](../../../docs/adr/0013-fail-closed-failure-semantics.md)).

use crate::{agent_message, field as read_field, some_or};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shiin_classify::{classify_command, path_effect};
use shiin_schema::{
    ActionRequest, Agent, Assurance, Confidence, Decision, DecisionValue, Effect, Host, Workspace,
    clock, versions,
};

/// The agent identifier this adapter reports.
pub const AGENT_ID: &str = "claude-code";

/// A `PreToolUse` payload from Claude Code.
#[derive(Debug, Deserialize)]
pub struct PreToolUse {
    /// The host session.
    #[serde(default)]
    pub session_id: String,
    /// The working directory the host is running in.
    #[serde(default)]
    pub cwd: String,
    /// The host's name for the tool, such as `Bash` or `Write`.
    #[serde(default)]
    pub tool_name: String,
    /// The tool's arguments, whose shape depends on the tool.
    #[serde(default)]
    pub tool_input: Value,
}

/// The response Claude Code reads back from a `PreToolUse` hook.
#[derive(Debug, Serialize)]
pub struct HookResponse {
    /// Wrapper the host requires.
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: HookOutput,
}

/// The decision Claude Code acts on.
#[derive(Debug, Serialize)]
pub struct HookOutput {
    /// Always `PreToolUse` for this hook.
    #[serde(rename = "hookEventName")]
    pub hook_event_name: String,
    /// `allow`, `deny`, or `ask`.
    #[serde(rename = "permissionDecision")]
    pub permission_decision: String,
    /// Text shown to the agent and the developer.
    #[serde(rename = "permissionDecisionReason")]
    pub permission_decision_reason: String,
}

impl HookResponse {
    /// Builds the response for a decision.
    ///
    /// `pending` becomes the host's native `ask` in v0.1, which is an interim step
    /// until the approval protocol exists in v0.3.
    #[must_use]
    pub fn from_decision(decision: &Decision) -> Self {
        let permission = match decision.decision {
            DecisionValue::Allow => "allow",
            DecisionValue::Deny => "deny",
            DecisionValue::Pending => "ask",
        };
        Self::new(permission, &agent_message(decision))
    }

    /// Builds a deny response, used when anything at all goes wrong.
    #[must_use]
    pub fn fail_closed(detail: &str) -> Self {
        Self::new(
            "deny",
            &format!("Shiin denied this action because it could not decide safely: {detail}"),
        )
    }

    fn new(permission: &str, reason: &str) -> Self {
        Self {
            hook_specific_output: HookOutput {
                hook_event_name: "PreToolUse".to_owned(),
                permission_decision: permission.to_owned(),
                permission_decision_reason: reason.to_owned(),
            },
        }
    }
}

/// Translates a `PreToolUse` payload into an action request.
///
/// An unrecognized tool becomes a vendor extension action type, which no rule in a
/// v0.1 policy can match, so it falls through to the default decision rather than
/// being allowed.
#[must_use]
pub fn to_action_request(payload: &PreToolUse, workspace_root: &str) -> ActionRequest {
    let (action_type, effects) = classify(payload, workspace_root);
    ActionRequest {
        schema: versions::ACTION_REQUEST.to_owned(),
        id: clock::new_id(),
        session: payload.session_id.clone(),
        agent: Agent {
            id: AGENT_ID.to_owned(),
            assurance: Assurance::Asserted,
        },
        action_type,
        effects,
        task: None,
        workspace: Workspace {
            root: workspace_root.to_owned(),
            vcs: None,
        },
        policy_snapshot: String::new(),
        host: Host {
            name: AGENT_ID.to_owned(),
            tool: Some(payload.tool_name.clone()),
        },
    }
}

/// Maps a host tool and its arguments to an action type and effects.
fn classify(payload: &PreToolUse, workspace_root: &str) -> (String, Vec<Effect>) {
    let field = |name: &str| read_field(&payload.tool_input, name);

    match payload.tool_name.as_str() {
        "Read" => (
            "fs.read".to_owned(),
            vec![path_effect(&field("file_path"), workspace_root)],
        ),
        "Write" | "Edit" | "NotebookEdit" => (
            "fs.write".to_owned(),
            vec![path_effect(&field("file_path"), workspace_root)],
        ),
        "Glob" | "Grep" => (
            "fs.list".to_owned(),
            vec![path_effect(
                &some_or(&field("path"), workspace_root),
                workspace_root,
            )],
        ),
        "Bash" => classify_command(&field("command"), workspace_root),
        "WebFetch" | "WebSearch" => (
            "net.fetch".to_owned(),
            vec![Effect {
                kind: "url".to_owned(),
                value: some_or(&field("url"), &field("query")),
                confidence: Confidence::Exact,
                action_type: None,
            }],
        ),
        other => (
            format!("x-claude-code.{}", other.to_lowercase()),
            vec![Effect {
                kind: "command".to_owned(),
                value: other.to_owned(),
                confidence: Confidence::Opaque,
                action_type: None,
            }],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A workspace root that exists on no platform.
    ///
    /// Canonicalization resolves real symlinks, so a root that happens to exist is not a
    /// fixed point: macOS rewrites `/home` through autofs.
    const ROOT: &str = "/shiin-test-root/project";

    fn payload(tool: &str, input: Value) -> PreToolUse {
        PreToolUse {
            session_id: "sess-1".to_owned(),
            cwd: ROOT.to_owned(),
            tool_name: tool.to_owned(),
            tool_input: input,
        }
    }

    #[test]
    fn a_write_becomes_a_path_effect() {
        let request = to_action_request(
            &payload("Write", serde_json::json!({"file_path": "src/login.rs"})),
            ROOT,
        );
        assert_eq!(request.action_type, "fs.write");
        assert_eq!(
            request.effects[0].value,
            "/shiin-test-root/project/src/login.rs"
        );
        assert_eq!(request.agent.assurance, Assurance::Asserted);
    }

    #[test]
    fn an_interpreter_command_is_opaque() {
        let request = to_action_request(
            &payload("Bash", serde_json::json!({"command": "python cleanup.py"})),
            ROOT,
        );
        assert_eq!(request.action_type, "shell.exec");
        assert!(request.has_opaque_effect());
    }

    #[test]
    fn an_unknown_tool_becomes_a_vendor_extension() {
        let request = to_action_request(&payload("SomeNewTool", Value::Null), ROOT);
        assert_eq!(request.action_type, "x-claude-code.somenewtool");
        assert!(request.has_opaque_effect());
    }

    #[test]
    fn a_pending_decision_asks_the_host() {
        let decision = Decision {
            schema: versions::DECISION.to_owned(),
            request_id: "r".to_owned(),
            decision: DecisionValue::Pending,
            reason: "pending.risk".to_owned(),
            stage: "risk".to_owned(),
            matched_rules: Vec::new(),
            policy_snapshot: "sha256:0".to_owned(),
            engine_version: "0".to_owned(),
        };
        let response = HookResponse::from_decision(&decision);
        assert_eq!(response.hook_specific_output.permission_decision, "ask");
    }

    #[test]
    fn a_failure_denies_and_says_why_without_leaking() {
        let response = HookResponse::fail_closed("policy is unreadable");
        assert_eq!(response.hook_specific_output.permission_decision, "deny");
    }
}
