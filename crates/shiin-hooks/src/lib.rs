//! Host hook adapters.
//!
//! Each host has its own payload and its own way of hearing a decision, so each gets a
//! module. What they share is the shape of the work: read the host's payload, build an
//! [action request](shiin_schema::ActionRequest), and turn a decision back into the
//! host's own protocol, where a failure has to become an explicit denial because every
//! one of these hosts treats a hook that does not answer as permission to proceed
//! ([ADR-0013](../../../docs/adr/0013-fail-closed-failure-semantics.md)).
//!
//! | Module | Host | Enforcement level | Verified against |
//! |---|---|---|---|
//! | [`claude_code`] | Claude Code | L1 cooperative | the hooks documentation, 2026-09-14 |
//! | [`codex`] | OpenAI Codex CLI | L1 cooperative | the hooks documentation, 2026-09-14, except its payload field names |
//! | [`cursor`] | Cursor | L1 cooperative | the hooks documentation, 2026-09-14 |

pub mod claude_code;
pub mod codex;
pub mod cursor;

use serde_json::Value;
use shiin_schema::Decision;

/// Reads a string field from a host's tool arguments.
#[must_use]
pub fn field(input: &Value, name: &str) -> String {
    input
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// The first value if it is not empty, otherwise the fallback.
#[must_use]
pub fn some_or(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

/// The message an agent is told.
///
/// It says what was refused and never reveals policy structure, other rules, or internal
/// errors, so it cannot be used to probe for a bypass (`DEC-009`).
#[must_use]
pub fn agent_message(decision: &Decision) -> String {
    match decision.decision {
        shiin_schema::DecisionValue::Allow => "Allowed by policy.".to_owned(),
        shiin_schema::DecisionValue::Deny => format!(
            "Denied by Shiin ({}). This action is outside what the current policy or task allows.",
            decision.reason
        ),
        shiin_schema::DecisionValue::Pending => format!(
            "Shiin needs a person to approve this action ({}).",
            decision.reason
        ),
    }
}
