//! The Cursor hook adapter.
//!
//! Cursor runs a hook per event rather than one hook for every tool, and reads back
//! `permission` with `allow`, `deny`, or `ask`
//! (verified against <https://cursor.com/docs/hooks> on 2026-09-14).
//!
//! Enforcement level L1, cooperative. Two details matter when installing it: Cursor
//! allows an action when a hook fails unless the hook is configured with
//! `failClosed: true`, and whether `ask` is honored for shell and tool events is not
//! verified, so a pending decision is reported as `ask` and also explained in the
//! message in case the host treats it as an allow.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use shiin_classify::{classify_command, path_effect};
use shiin_schema::{
    ActionRequest, Agent, Assurance, Confidence, Decision, DecisionValue, Effect, Host, Workspace,
    clock, versions,
};

use crate::{agent_message, field as read_field, some_or};

/// The agent identifier this adapter reports.
pub const AGENT_ID: &str = "cursor";

/// A hook payload from Cursor.
#[derive(Debug, Default, Deserialize)]
pub struct HookPayload {
    /// Which event fired, such as `beforeShellExecution`.
    #[serde(default)]
    pub hook_event_name: String,
    /// The conversation this belongs to.
    #[serde(default)]
    pub conversation_id: String,
    /// The workspace directories the host has open.
    #[serde(default)]
    pub workspace_roots: Vec<String>,
    /// The working directory, for shell events.
    #[serde(default)]
    pub cwd: String,
    /// The command, for shell events.
    #[serde(default)]
    pub command: String,
    /// The path, for file events.
    #[serde(default)]
    pub file_path: String,
    /// The tool name, for tool events.
    #[serde(default)]
    pub tool_name: String,
    /// The tool's arguments, for tool events.
    #[serde(default)]
    pub tool_input: Value,
}

impl HookPayload {
    /// The workspace this payload applies to, preferring the host's own roots.
    #[must_use]
    pub fn workspace(&self, fallback: &str) -> String {
        self.workspace_roots
            .first()
            .filter(|root| !root.is_empty())
            .cloned()
            .unwrap_or_else(|| some_or(&self.cwd, fallback))
    }
}

/// The response Cursor reads back.
#[derive(Debug, Serialize)]
pub struct HookResponse {
    /// `allow`, `deny`, or `ask`.
    pub permission: String,
    /// Text shown to the person using Cursor.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub user_message: String,
    /// Text given to the agent.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub agent_message: String,
}

impl HookResponse {
    /// Builds the response for a decision.
    #[must_use]
    pub fn from_decision(decision: &Decision) -> Self {
        let permission = match decision.decision {
            DecisionValue::Allow => "allow",
            DecisionValue::Deny => "deny",
            DecisionValue::Pending => "ask",
        };
        let message = agent_message(decision);
        Self {
            permission: permission.to_owned(),
            user_message: message.clone(),
            agent_message: message,
        }
    }

    /// Builds a deny response, used when anything at all goes wrong.
    #[must_use]
    pub fn fail_closed(detail: &str) -> Self {
        let message =
            format!("Shiin denied this action because it could not decide safely: {detail}");
        Self {
            permission: "deny".to_owned(),
            user_message: message.clone(),
            agent_message: message,
        }
    }
}

/// Translates a Cursor payload into an action request.
#[must_use]
pub fn to_action_request(payload: &HookPayload, workspace_root: &str) -> ActionRequest {
    let (action_type, effects) = classify(payload, workspace_root);
    ActionRequest {
        schema: versions::ACTION_REQUEST.to_owned(),
        id: clock::new_id(),
        session: payload.conversation_id.clone(),
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
            tool: Some(payload.hook_event_name.clone()),
        },
    }
}

/// Maps a Cursor event to an action type and effects.
fn classify(payload: &HookPayload, workspace_root: &str) -> (String, Vec<Effect>) {
    match payload.hook_event_name.as_str() {
        "beforeShellExecution" => classify_command(&payload.command, workspace_root),
        "beforeReadFile" => (
            "fs.read".to_owned(),
            vec![path_effect(&payload.file_path, workspace_root)],
        ),
        "beforeMCPExecution" => (
            format!("x-cursor.mcp.{}", payload.tool_name.to_lowercase()),
            vec![Effect {
                kind: "command".to_owned(),
                value: some_or(
                    &read_field(&payload.tool_input, "command"),
                    &payload.tool_name,
                ),
                // A tool server describes its own behavior, which Shiin cannot verify.
                confidence: Confidence::Opaque,
                action_type: None,
            }],
        ),
        other => (
            format!("x-cursor.{}", other.to_lowercase()),
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

    #[test]
    fn a_shell_event_becomes_a_command() {
        let payload = HookPayload {
            hook_event_name: "beforeShellExecution".to_owned(),
            command: "cat .env".to_owned(),
            workspace_roots: vec!["/home/dev/project".to_owned()],
            ..HookPayload::default()
        };
        let request = to_action_request(&payload, &payload.workspace("/fallback"));
        assert_eq!(request.action_type, "shell.exec");
        assert!(
            request
                .effects
                .iter()
                .any(|effect| effect.action_type.as_deref() == Some("fs.read")),
            "the read the command performs is recorded"
        );
    }

    #[test]
    fn a_file_read_event_becomes_a_path() {
        let payload = HookPayload {
            hook_event_name: "beforeReadFile".to_owned(),
            file_path: ".env".to_owned(),
            ..HookPayload::default()
        };
        let request = to_action_request(&payload, "/home/dev/project");
        assert_eq!(request.action_type, "fs.read");
        assert_eq!(request.effects[0].value, "/home/dev/project/.env");
    }

    #[test]
    fn a_tool_server_call_is_opaque() {
        let payload = HookPayload {
            hook_event_name: "beforeMCPExecution".to_owned(),
            tool_name: "deploy".to_owned(),
            ..HookPayload::default()
        };
        let request = to_action_request(&payload, "/home/dev/project");
        assert_eq!(request.action_type, "x-cursor.mcp.deploy");
        assert!(request.has_opaque_effect());
    }

    #[test]
    fn the_host_workspace_wins_over_the_fallback() {
        let payload = HookPayload {
            workspace_roots: vec!["/home/dev/project".to_owned()],
            cwd: "/somewhere/else".to_owned(),
            ..HookPayload::default()
        };
        assert_eq!(payload.workspace("/fallback"), "/home/dev/project");
    }

    #[test]
    fn a_failure_denies() {
        assert_eq!(HookResponse::fail_closed("no policy").permission, "deny");
    }
}
