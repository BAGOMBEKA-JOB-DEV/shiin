//! Spike for ADR-0014: can Cedar express Shiin's decision semantics?
//!
//! This is not product code and nothing here is normative.
//! It compiles each specification test vector's policy into Cedar, evaluates it,
//! applies the combining rules from the evaluation specification, and compares the
//! outcome with the vector's expectation.
//!
//! Run it with `cargo run -p spike-cedar`.

#![expect(
    clippy::print_stdout,
    reason = "the spike reports its findings to the terminal"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cedar_policy::{
    Authorizer, Context, Decision, Entities, Policy, PolicyId, PolicySet, Request,
    RestrictedExpression,
};
use globset::GlobBuilder;
use serde_json::Value;

/// The default threshold for `bulk_delete`, used when the policy entry gives none.
const DEFAULT_BULK_DELETE_THRESHOLD: u64 = 20;

/// One stage's contribution to a decision.
enum StageResult {
    Deny(&'static str),
    Pending(&'static str),
    Allow,
    Pass,
}

fn main() -> ExitCode {
    let vectors = match load_vectors() {
        Ok(vectors) => vectors,
        Err(error) => {
            println!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "{:<8} {:<30} {:<26} {:<26} result",
        "vector", "name", "expected", "actual"
    );
    let mut failures = 0_usize;
    for (path, vector) in &vectors {
        let id = vector["id"].as_str().unwrap_or("?");
        let name = vector["name"].as_str().unwrap_or("?");
        let expected = format!(
            "{} / {}",
            vector["expected"]["decision"].as_str().unwrap_or("?"),
            vector["expected"]["reason"].as_str().unwrap_or("?")
        );
        match evaluate(vector) {
            Ok((decision, reason)) => {
                let actual = format!("{decision} / {reason}");
                let ok = actual == expected;
                if !ok {
                    failures += 1;
                }
                println!(
                    "{id:<8} {name:<30} {expected:<26} {actual:<26} {}",
                    if ok { "match" } else { "MISMATCH" }
                );
            }
            Err(error) => {
                failures += 1;
                let actual = "-";
                println!("{id:<8} {name:<30} {expected:<26} {actual:<26} ERROR: {error}");
                println!("         ({})", path.display());
            }
        }
    }

    println!();
    if failures == 0 {
        println!(
            "all {} vectors reproduced: Cedar expresses the semantics",
            vectors.len()
        );
        ExitCode::SUCCESS
    } else {
        println!("{failures} of {} vectors did not reproduce", vectors.len());
        ExitCode::FAILURE
    }
}

fn load_vectors() -> Result<Vec<(PathBuf, Value)>, String> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/test-vectors");
    let entries =
        fs::read_dir(&directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    paths.sort();

    let mut vectors = Vec::new();
    for path in paths {
        let text =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        vectors.push((path, value));
    }
    Ok(vectors)
}

/// Evaluates one vector and returns its decision and reason code.
fn evaluate(vector: &Value) -> Result<(String, String), String> {
    let request = &vector["action_request"];
    let policy = &vector["policy"];
    let action_type = request["action_type"]
        .as_str()
        .ok_or("the action request has no action_type")?;
    let empty = Vec::new();
    let effects = request["effects"].as_array().unwrap_or(&empty);
    let agent = request["agent"]["id"].as_str().unwrap_or("");
    let workspace = request["workspace"]["root"]
        .as_str()
        .ok_or("the action request has no workspace root")?;

    let matched = matched_rules(policy, action_type, effects, agent, workspace);
    let policy_stage = policy_stage(policy, action_type, agent, &matched)?;
    let intent_stage = intent_stage(vector, action_type, effects, workspace);
    let risk_stage = risk_stage(policy, effects);

    // Combining, in the order the evaluation specification fixes.
    let stages = [&policy_stage, &intent_stage, &risk_stage];
    let denied = stages.iter().find_map(|stage| match stage {
        StageResult::Deny(code) => Some(*code),
        _ => None,
    });
    let held = stages.iter().find_map(|stage| match stage {
        StageResult::Pending(code) => Some(*code),
        _ => None,
    });

    let (decision, reason) = if let Some(code) = denied {
        ("deny", code)
    } else if let Some(code) = held {
        // An unattended engine cannot hold an action for a human.
        if vector["mode"].as_str() == Some("unattended") {
            ("deny", "deny.unattended")
        } else {
            ("pending", code)
        }
    } else if matches!(policy_stage, StageResult::Allow) {
        ("allow", "allow.policy")
    } else {
        ("deny", "deny.default")
    };
    Ok((decision.to_owned(), reason.to_owned()))
}

/// Rule identifiers whose matchers the request satisfies, computed in Rust because
/// Cedar's `like` operator has only one wildcard and cannot express path globs.
fn matched_rules(
    policy: &Value,
    action_type: &str,
    effects: &[Value],
    agent: &str,
    workspace: &str,
) -> Vec<String> {
    let mut matched = Vec::new();
    for rule in rules(policy) {
        let Some(id) = rule["id"].as_str() else {
            continue;
        };
        if !list_matches(&rule["action_types"], |pattern| {
            action_type_matches(pattern, action_type)
        }) {
            continue;
        }
        if !rule["agents"].is_null() && !list_matches(&rule["agents"], |value| value == agent) {
            continue;
        }
        if !rule["resources"].is_null()
            && !list_matches(&rule["resources"], |pattern| {
                effect_matches(effects, "path", pattern, workspace)
            })
        {
            continue;
        }
        if !rule["commands"].is_null()
            && !list_matches(&rule["commands"], |pattern| {
                effect_matches(effects, "command", pattern, workspace)
            })
        {
            continue;
        }
        matched.push(id.to_owned());
    }
    matched
}

/// Runs the policy stage through Cedar and maps its answer onto Shiin's values.
fn policy_stage(
    policy: &Value,
    action_type: &str,
    agent: &str,
    matched: &[String],
) -> Result<StageResult, String> {
    let mut set = PolicySet::new();
    let mut pending_rules = Vec::new();
    for rule in rules(policy) {
        let Some(id) = rule["id"].as_str() else {
            continue;
        };
        let effect = rule["effect"].as_str().unwrap_or("deny");
        let keyword = if effect == "deny" { "forbid" } else { "permit" };
        let annotation = if effect == "pending" {
            pending_rules.push(id.to_owned());
            "@decision(\"pending\")\n"
        } else {
            ""
        };
        let source = format!(
            "{annotation}permit_or_forbid_placeholder(principal, action, resource)\nwhen {{ context.matched_rules.contains(\"{id}\") }};"
        )
        .replace("permit_or_forbid_placeholder", keyword);
        let parsed = Policy::parse(Some(PolicyId::new(id)), &source)
            .map_err(|error| format!("rule `{id}` does not compile to Cedar: {error}"))?;
        set.add(parsed)
            .map_err(|error| format!("rule `{id}` cannot be added: {error}"))?;
    }
    if set.is_empty() {
        return Ok(StageResult::Pass);
    }

    let principal = format!("Shiin::Agent::\"{agent}\"")
        .parse()
        .map_err(|error| format!("principal: {error}"))?;
    let action = format!("Shiin::Action::\"{action_type}\"")
        .parse()
        .map_err(|error| format!("action: {error}"))?;
    let resource = "Shiin::Resource::\"request\""
        .parse()
        .map_err(|error| format!("resource: {error}"))?;
    let context = Context::from_pairs([(
        "matched_rules".to_owned(),
        RestrictedExpression::new_set(
            matched
                .iter()
                .map(|id| RestrictedExpression::new_string(id.clone())),
        ),
    )])
    .map_err(|error| format!("context: {error}"))?;
    let request = Request::new(principal, action, resource, context, None)
        .map_err(|error| format!("request: {error}"))?;

    let response = Authorizer::new().is_authorized(&request, &set, &Entities::empty());
    let determining: Vec<String> = response
        .diagnostics()
        .reason()
        .map(ToString::to_string)
        .collect();

    match response.decision() {
        // Cedar's forbid overrides permit, which is exactly "deny always wins".
        Decision::Deny if !determining.is_empty() => Ok(StageResult::Deny("deny.policy")),
        // No permit matched, so the policy stage neither allows nor denies.
        Decision::Deny => Ok(StageResult::Pass),
        Decision::Allow if determining.iter().any(|id| pending_rules.contains(id)) => {
            Ok(StageResult::Pending("pending.grantable"))
        }
        Decision::Allow => Ok(StageResult::Allow),
    }
}

/// Checks the action against the active task declaration.
fn intent_stage(
    vector: &Value,
    action_type: &str,
    effects: &[Value],
    workspace: &str,
) -> StageResult {
    let task = &vector["task"];
    if !task.is_object() {
        return StageResult::Pass;
    }
    if list_matches(&task["forbid"]["action_types"], |pattern| {
        action_type_matches(pattern, action_type)
    }) {
        return StageResult::Deny("deny.intent");
    }
    if list_matches(&task["require_pending_for"], |pattern| {
        action_type_matches(pattern, action_type)
    }) {
        return StageResult::Pending("pending.intent");
    }
    if let Some(scope) = task["scope"]["paths"].as_array() {
        let paths: Vec<&str> = effects
            .iter()
            .filter(|effect| effect["kind"].as_str() == Some("path"))
            .filter_map(|effect| effect["value"].as_str())
            .collect();
        let outside = paths.iter().any(|path| {
            !scope
                .iter()
                .filter_map(|pattern| pattern.as_str())
                .any(|pattern| glob_matches(pattern, path, workspace))
        });
        if outside && !paths.is_empty() {
            return StageResult::Deny("deny.intent");
        }
    }
    StageResult::Pass
}

/// Raises the configured risk signals for this request.
fn risk_stage(policy: &Value, effects: &[Value]) -> StageResult {
    let entry = |signal: &str| policy["risk"].get(signal).cloned();
    let configured = |signal: &str| {
        entry(signal).and_then(|value| match &value {
            Value::String(decision) => Some(decision.clone()),
            Value::Object(_) => value["decision"].as_str().map(ToOwned::to_owned),
            _ => None,
        })
    };
    let threshold = |signal: &str| {
        entry(signal)
            .and_then(|value| value["threshold"].as_u64())
            .unwrap_or(DEFAULT_BULK_DELETE_THRESHOLD)
    };
    let mut raised: Vec<(&'static str, String)> = Vec::new();

    if effects
        .iter()
        .any(|effect| effect["confidence"].as_str() == Some("opaque"))
        && let Some(decision) = configured("opaque_command")
    {
        raised.push(("pending.opaque", decision));
    }
    let deleted = effects
        .iter()
        .filter(|effect| effect["kind"].as_str() == Some("count"))
        .filter_map(|effect| effect["value"].as_str())
        .filter_map(|value| value.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    if deleted > threshold("bulk_delete")
        && let Some(decision) = configured("bulk_delete")
    {
        raised.push(("pending.risk", decision));
    }
    if effects.iter().any(|effect| {
        effect["kind"].as_str() == Some("command")
            && effect["value"].as_str().is_some_and(is_destructive_vcs)
    }) && let Some(decision) = configured("destructive_vcs")
    {
        raised.push(("pending.risk", decision));
    }

    if raised.iter().any(|(_, decision)| decision == "deny") {
        return StageResult::Deny("deny.risk");
    }
    if let Some((code, _)) = raised.iter().find(|(_, decision)| decision == "pending") {
        return StageResult::Pending(code);
    }
    StageResult::Pass
}

fn is_destructive_vcs(command: &str) -> bool {
    ["--force", "-f ", "reset --hard", "branch -D"]
        .iter()
        .any(|marker| command.contains(marker))
        && command.starts_with("git")
}

fn rules(policy: &Value) -> Vec<&Value> {
    policy["layers"]
        .as_array()
        .map(|layers| {
            layers
                .iter()
                .filter_map(|layer| layer["rules"].as_array())
                .flatten()
                .collect()
        })
        .unwrap_or_default()
}

/// Whether any entry of a JSON string array satisfies the predicate.
fn list_matches(list: &Value, predicate: impl Fn(&str) -> bool) -> bool {
    list.as_array()
        .is_some_and(|values| values.iter().filter_map(Value::as_str).any(predicate))
}

fn effect_matches(effects: &[Value], kind: &str, pattern: &str, workspace: &str) -> bool {
    effects
        .iter()
        .filter(|effect| effect["kind"].as_str() == Some(kind))
        .filter_map(|effect| effect["value"].as_str())
        .any(|value| glob_matches(pattern, value, workspace))
}

fn action_type_matches(pattern: &str, action_type: &str) -> bool {
    pattern
        .strip_suffix('*')
        .map_or(pattern == action_type, |prefix| {
            action_type.starts_with(prefix)
        })
}

/// Matches a policy glob against a value.
///
/// A workspace-relative pattern resolves against the request's workspace root, so it
/// can never match outside the workspace. A home-relative pattern is left to match
/// anywhere, because home expansion happens when policy is compiled (ADR-0019).
fn glob_matches(pattern: &str, value: &str, workspace: &str) -> bool {
    let rewritten = if let Some(rest) = pattern.strip_prefix("./") {
        format!("{}/{rest}", workspace.trim_end_matches('/'))
    } else if let Some(rest) = pattern.strip_prefix("~/") {
        format!("**/{rest}")
    } else {
        pattern.to_owned()
    };
    GlobBuilder::new(&rewritten)
        .literal_separator(false)
        .build()
        .is_ok_and(|glob| glob.compile_matcher().is_match(value))
}
