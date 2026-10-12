//! Runs policy cases in the test-vector format.
//!
//! The project's conformance corpus and `shiin policy test` are the same thing: a case
//! names an action request, the policy it is evaluated against, and the decision it
//! expects. Sharing one runner means an operator's cases are checked exactly the way the
//! specification's are.
//!
//! The format is documented in `docs/spec/test-vectors/README.md`.

use serde_json::Value;
use shiin_policy::CompiledPolicy;
use shiin_schema::{ActionRequest, Mode, Policy, TaskDeclaration, TaskUsage};

use crate::{Inputs, decide};

/// What a case expected, and what it produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The case's identifier.
    pub id: String,
    /// What the case says should happen, as a decision and a reason code.
    pub expected: (String, String),
    /// What the engine decided.
    pub actual: (String, String),
}

impl Outcome {
    /// Whether the engine agreed with the case.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.expected == self.actual
    }
}

/// Runs one case.
///
/// `fallback` supplies the policy for a case that does not carry one, which lets an
/// operator write cases against the policy their workspace already has.
///
/// # Errors
///
/// Returns an error if the case or its policy cannot be read, or the policy cannot be
/// compiled.
pub fn run(case: &Value, fallback: Option<&Policy>) -> Result<Outcome, String> {
    let id = case["id"].as_str().unwrap_or("case").to_owned();
    let request: ActionRequest = serde_json::from_value(case["action_request"].clone())
        .map_err(|error| format!("the action request does not parse: {error}"))?;

    let source: Policy = match (case.get("policy"), fallback) {
        (Some(value), _) if value.is_object() => serde_json::from_value(value.clone())
            .map_err(|error| format!("the policy does not parse: {error}"))?,
        (_, Some(policy)) => policy.clone(),
        _ => return Err("the case has no policy, and none was supplied".to_owned()),
    };

    let task: Option<TaskDeclaration> = match case.get("task") {
        Some(value) if value.is_object() => Some(
            serde_json::from_value(value.clone())
                .map_err(|error| format!("the task does not parse: {error}"))?,
        ),
        _ => None,
    };
    let usage: TaskUsage = match case.get("usage") {
        Some(value) if value.is_object() => serde_json::from_value(value.clone())
            .map_err(|error| format!("the usage does not parse: {error}"))?,
        _ => TaskUsage::default(),
    };
    let mode = if case["mode"].as_str() == Some("unattended") {
        Mode::Unattended
    } else {
        Mode::Attended
    };
    // A case without a time is evaluated before any expiry it declares.
    let now = case["now"].as_str().unwrap_or("1970-01-01T00:00:00Z");

    let compiled = CompiledPolicy::compile(&source, &request.workspace.root)
        .map_err(|error| format!("the policy does not compile: {error}"))?;
    let decision = decide(&Inputs {
        request: &request,
        policy: &compiled,
        source: &source,
        task: task.as_ref(),
        usage,
        now,
        mode,
    });

    let actual = serde_json::to_value(decision.decision)
        .map_err(|error| format!("the decision does not serialize: {error}"))?;
    let actual = actual
        .as_str()
        .ok_or_else(|| "a decision value is a string".to_owned())?
        .to_owned();

    Ok(Outcome {
        id,
        expected: (
            case["expected"]["decision"]
                .as_str()
                .unwrap_or("?")
                .to_owned(),
            case["expected"]["reason"]
                .as_str()
                .unwrap_or("?")
                .to_owned(),
        ),
        actual: (actual, decision.reason),
    })
}
