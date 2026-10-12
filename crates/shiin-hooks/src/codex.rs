//! The `OpenAI` Codex CLI hook adapter.
//!
//! Codex runs a `PreToolUse` hook and reads a decision back in the same shape Claude Code
//! uses, including `hookSpecificOutput.permissionDecision`
//! (verified against <https://learn.chatgpt.com/docs/hooks> on 2026-09-14).
//!
//! Enforcement level L1, cooperative, and weaker than it first appears: Codex requires a
//! hook to be trusted before it runs, it does not intercept hosted tools such as its web
//! search, and its own documentation calls hooks "a useful guardrail, not a complete
//! enforcement boundary".
//!
//! The payload field names were not verified, so this adapter accepts the names Claude
//! Code uses and the aliases Codex is likely to use, and refuses to guess beyond that: an
//! unrecognized tool becomes a vendor extension action type that no rule can match.

use serde::Deserialize;
use serde_json::Value;
use shiin_classify::{classify_command, path_effect};
use shiin_schema::{
    ActionRequest, Agent, Assurance, Confidence, Effect, Host, Workspace, clock, versions,
};

use crate::{field as read_field, some_or};

/// The agent identifier this adapter reports.
pub const AGENT_ID: &str = "codex-cli";

/// A `PreToolUse` payload from Codex.
#[derive(Debug, Default, Deserialize)]
pub struct PreToolUse {
    /// The host session.
    #[serde(default, alias = "sessionId", alias = "conversation_id")]
    pub session_id: String,
    /// The working directory, when the host reports one.
    #[serde(default, alias = "workspace_root", alias = "working_directory")]
    pub cwd: String,
    /// The tool's name, such as `Bash` or `apply_patch`.
    #[serde(default, alias = "toolName", alias = "name")]
    pub tool_name: String,
    /// The tool's arguments.
    #[serde(default, alias = "toolInput", alias = "arguments")]
    pub tool_input: Value,
}

/// Translates a Codex payload into an action request.
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

/// Maps a Codex tool to an action type and effects.
fn classify(payload: &PreToolUse, workspace_root: &str) -> (String, Vec<Effect>) {
    let field = |name: &str| read_field(&payload.tool_input, name);

    match payload.tool_name.as_str() {
        // Codex edits files by applying a patch rather than writing one path at a time.
        "apply_patch" | "Edit" | "Write" => {
            let path = some_or(&field("path"), &field("file_path"));
            if path.is_empty() {
                // A patch with no readable path still changes files, so it is opaque
                // rather than harmless.
                (
                    "fs.write".to_owned(),
                    vec![Effect {
                        kind: "command".to_owned(),
                        value: "apply_patch".to_owned(),
                        confidence: Confidence::Opaque,
                        action_type: None,
                    }],
                )
            } else {
                (
                    "fs.write".to_owned(),
                    vec![path_effect(&path, workspace_root)],
                )
            }
        }
        "Read" => (
            "fs.read".to_owned(),
            vec![path_effect(
                &some_or(&field("path"), &field("file_path")),
                workspace_root,
            )],
        ),
        "Bash" | "shell" | "local_shell" => {
            classify_command(&some_or(&field("command"), &field("cmd")), workspace_root)
        }
        other => (
            format!("x-codex.{}", other.to_lowercase()),
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

    fn payload(tool: &str, input: Value) -> PreToolUse {
        PreToolUse {
            session_id: "sess-1".to_owned(),
            cwd: "/home/dev/project".to_owned(),
            tool_name: tool.to_owned(),
            tool_input: input,
        }
    }

    #[test]
    fn a_patch_with_a_path_becomes_a_write() {
        let request = to_action_request(
            &payload("apply_patch", serde_json::json!({"path": "src/login.rs"})),
            "/home/dev/project",
        );
        assert_eq!(request.action_type, "fs.write");
        assert_eq!(request.effects[0].value, "/home/dev/project/src/login.rs");
    }

    #[test]
    fn a_patch_without_a_readable_path_is_opaque() {
        let request = to_action_request(
            &payload("apply_patch", serde_json::json!({"patch": "@@ -1 +1 @@"})),
            "/home/dev/project",
        );
        assert!(
            request.has_opaque_effect(),
            "an unreadable patch is not harmless"
        );
    }

    #[test]
    fn alternative_field_names_are_accepted() {
        let parsed: PreToolUse = serde_json::from_str(
            r#"{"sessionId":"s","name":"shell","arguments":{"cmd":"cargo test"}}"#,
        )
        .expect("the aliases parse");
        let request = to_action_request(&parsed, "/home/dev/project");
        assert_eq!(request.action_type, "shell.exec");
        assert_eq!(request.host.tool.as_deref(), Some("shell"));
    }

    #[test]
    fn an_unknown_tool_becomes_a_vendor_extension() {
        let request = to_action_request(&payload("web_search", Value::Null), "/home/dev/project");
        assert_eq!(request.action_type, "x-codex.web_search");
    }
}
