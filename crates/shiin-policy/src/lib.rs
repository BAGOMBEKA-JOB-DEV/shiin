//! Compiles a Shiin policy into Cedar and evaluates the policy stage.
//!
//! A compiled policy is bound to one workspace root, because workspace-relative
//! globs resolve against it (`AR-009`). Globs are matched in Rust and reach Cedar
//! as `context.matched_rules`, since Cedar's `like` cannot express `**`
//! ([ADR-0014](../../../docs/adr/0014-cedar-policy-engine-with-toml-front-end.md)).

pub mod toml_front_end;

use std::collections::BTreeSet;

use cedar_policy::{
    Authorizer, Context, Decision as CedarDecision, Entities, Policy as CedarPolicy, PolicyId,
    PolicySet, Request as CedarRequest, RestrictedExpression,
};
use globset::{Glob, GlobMatcher};
use shiin_schema::{ActionRequest, DecisionValue, Effect, Policy, Rule};
use thiserror::Error;

/// Why a policy could not be compiled or evaluated.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PolicyError {
    /// A rule's glob is not valid.
    #[error("rule `{rule}` has an invalid pattern `{pattern}`: {source}")]
    Pattern {
        /// The offending rule.
        rule: String,
        /// The offending pattern.
        pattern: String,
        /// The underlying glob error.
        #[source]
        source: globset::Error,
    },
    /// Two rules share an identifier.
    #[error("rule `{0}` is defined more than once")]
    DuplicateRule(String),
    /// A rule could not be expressed in Cedar.
    #[error("rule `{rule}` does not compile to Cedar: {message}")]
    Cedar {
        /// The offending rule.
        rule: String,
        /// What Cedar reported.
        message: String,
    },
    /// The request could not be expressed as a Cedar request.
    #[error("cannot build a Cedar request: {0}")]
    Request(String),
    /// The policy file could not be read.
    #[error("the policy is not valid: {message}")]
    Syntax {
        /// What the parser reported.
        message: String,
    },
}

impl PolicyError {
    /// The stable code for this error, as registered in `docs/reference/error-codes.md`.
    ///
    /// Codes are part of the interface: a script may match on them, so they are never
    /// renumbered or reused.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match *self {
            // An unknown key is distinguished from other syntax errors, because it is
            // the common mistake and it fails closed rather than being ignored.
            Self::Syntax { ref message } if message.contains("unknown field") => "SHIIN-POL-0003",
            Self::Syntax { .. } => "SHIIN-POL-0002",
            Self::DuplicateRule(_) => "SHIIN-POL-0004",
            Self::Pattern { .. } => "SHIIN-POL-0005",
            Self::Cedar { .. } => "SHIIN-POL-0006",
            Self::Request(_) => "SHIIN-GEN-0001",
        }
    }
}

/// What the policy stage concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyOutcome {
    /// A rule allows the action.
    Allow(Vec<String>),
    /// A rule denies the action.
    Deny(Vec<String>),
    /// A rule holds the action for a human.
    Pending(Vec<String>),
    /// No rule matched, so the stage neither allows nor denies.
    NoMatch,
}

/// One rule with its patterns compiled.
struct CompiledRule {
    id: String,
    action_types: Vec<String>,
    agents: Vec<String>,
    resources: Vec<GlobMatcher>,
    commands: Vec<GlobMatcher>,
}

impl CompiledRule {
    /// Whether the request satisfies every matcher this rule declares.
    ///
    /// A rule matches either the request's own action type, or the action type of one
    /// of its effects together with that effect's resource (`AR-011`). That is what
    /// makes a rule written for `fs.read` cover `cat .env`, whose request type is
    /// `shell.exec`.
    fn matches(&self, request: &ActionRequest) -> bool {
        if self.matches_as(request, &request.action_type, None) {
            return true;
        }
        request.effects.iter().any(|effect| {
            effect
                .action_type
                .as_deref()
                .is_some_and(|action_type| self.matches_as(request, action_type, Some(effect)))
        })
    }

    /// Whether the rule matches for one action type, optionally scoped to one effect.
    fn matches_as(
        &self,
        request: &ActionRequest,
        action_type: &str,
        effect: Option<&Effect>,
    ) -> bool {
        if !self
            .action_types
            .iter()
            .any(|pattern| action_type_matches(pattern, action_type))
        {
            return false;
        }
        if !self.agents.is_empty() && !self.agents.contains(&request.agent.id) {
            return false;
        }
        if !self.resources.is_empty() && !self.matches_paths(request, effect) {
            return false;
        }
        if !self.commands.is_empty()
            && !request
                .values_of("command")
                .any(|value| self.commands.iter().any(|glob| glob.is_match(value)))
        {
            return false;
        }
        true
    }

    /// Whether the rule's path globs match, scoped to one effect when given.
    fn matches_paths(&self, request: &ActionRequest, effect: Option<&Effect>) -> bool {
        match effect {
            // An effect-scoped match only considers that effect's own path, so a rule
            // cannot be satisfied by an unrelated path in the same request.
            Some(effect) => {
                effect.is("path")
                    && self
                        .resources
                        .iter()
                        .any(|glob| glob.is_match(&effect.value))
            }
            None => request
                .values_of("path")
                .any(|value| self.resources.iter().any(|glob| glob.is_match(value))),
        }
    }
}

/// A policy compiled for one workspace, ready to evaluate.
pub struct CompiledPolicy {
    rules: Vec<CompiledRule>,
    pending: BTreeSet<String>,
    set: PolicySet,
    default: DecisionValue,
}

impl CompiledPolicy {
    /// Compiles `policy` for the workspace rooted at `workspace_root`.
    ///
    /// # Errors
    ///
    /// Returns an error if a rule has a duplicate identifier, an invalid glob, or
    /// cannot be expressed in Cedar.
    pub fn compile(policy: &Policy, workspace_root: &str) -> Result<Self, PolicyError> {
        let mut rules = Vec::new();
        let mut pending = BTreeSet::new();
        let mut identifiers = BTreeSet::new();
        let mut set = PolicySet::new();

        for rule in policy.rules() {
            if !identifiers.insert(rule.id.clone()) {
                return Err(PolicyError::DuplicateRule(rule.id.clone()));
            }
            if rule.effect == DecisionValue::Pending {
                pending.insert(rule.id.clone());
            }
            set.add(cedar_policy_for(rule)?)
                .map_err(|error| PolicyError::Cedar {
                    rule: rule.id.clone(),
                    message: error.to_string(),
                })?;
            rules.push(CompiledRule {
                id: rule.id.clone(),
                action_types: rule.action_types.clone(),
                agents: rule.agents.clone(),
                resources: compile_globs(rule, &rule.resources, workspace_root)?,
                commands: compile_globs(rule, &rule.commands, workspace_root)?,
            });
        }
        Ok(Self {
            rules,
            pending,
            set,
            default: policy.default,
        })
    }

    /// The decision to use when no stage produced one.
    #[must_use]
    pub fn default_decision(&self) -> DecisionValue {
        self.default
    }

    /// Identifiers of the rules whose matchers the request satisfies.
    #[must_use]
    pub fn matched_rules(&self, request: &ActionRequest) -> Vec<String> {
        self.rules
            .iter()
            .filter(|rule| rule.matches(request))
            .map(|rule| rule.id.clone())
            .collect()
    }

    /// Evaluates the policy stage for `request`.
    ///
    /// # Errors
    ///
    /// Returns an error if the request cannot be expressed as a Cedar request.
    pub fn evaluate(&self, request: &ActionRequest) -> Result<PolicyOutcome, PolicyError> {
        if self.rules.is_empty() {
            return Ok(PolicyOutcome::NoMatch);
        }
        let matched = self.matched_rules(request);
        let cedar_request = build_request(request, &matched)?;
        let response =
            Authorizer::new().is_authorized(&cedar_request, &self.set, &Entities::empty());
        let determining: Vec<String> = response
            .diagnostics()
            .reason()
            .map(ToString::to_string)
            .collect();

        Ok(match response.decision() {
            // Cedar's forbid overrides permit, which is "deny always wins".
            CedarDecision::Deny if !determining.is_empty() => PolicyOutcome::Deny(determining),
            CedarDecision::Deny => PolicyOutcome::NoMatch,
            CedarDecision::Allow if determining.iter().any(|id| self.pending.contains(id)) => {
                PolicyOutcome::Pending(determining)
            }
            CedarDecision::Allow => PolicyOutcome::Allow(determining),
        })
    }
}

/// Builds the Cedar policy for one rule, keeping the rule's identifier.
fn cedar_policy_for(rule: &Rule) -> Result<CedarPolicy, PolicyError> {
    let keyword = match rule.effect {
        DecisionValue::Deny => "forbid",
        DecisionValue::Allow | DecisionValue::Pending => "permit",
    };
    let annotation = if rule.effect == DecisionValue::Pending {
        "@decision(\"pending\")\n"
    } else {
        ""
    };
    let source = format!(
        "{annotation}{keyword}(principal, action, resource)\nwhen {{ context.matched_rules.contains(\"{}\") }};",
        rule.id
    );
    CedarPolicy::parse(Some(PolicyId::new(&rule.id)), source).map_err(|error| PolicyError::Cedar {
        rule: rule.id.clone(),
        message: error.to_string(),
    })
}

/// Expresses the request and its matched rules as a Cedar request.
fn build_request(request: &ActionRequest, matched: &[String]) -> Result<CedarRequest, PolicyError> {
    let principal = format!("Shiin::Agent::\"{}\"", request.agent.id)
        .parse()
        .map_err(|error: cedar_policy::ParseErrors| PolicyError::Request(error.to_string()))?;
    let action = format!("Shiin::Action::\"{}\"", request.action_type)
        .parse()
        .map_err(|error: cedar_policy::ParseErrors| PolicyError::Request(error.to_string()))?;
    let resource = "Shiin::Resource::\"request\""
        .parse()
        .map_err(|error: cedar_policy::ParseErrors| PolicyError::Request(error.to_string()))?;
    let context = Context::from_pairs([(
        "matched_rules".to_owned(),
        RestrictedExpression::new_set(
            matched
                .iter()
                .map(|id| RestrictedExpression::new_string(id.clone())),
        ),
    )])
    .map_err(|error| PolicyError::Request(error.to_string()))?;
    CedarRequest::new(principal, action, resource, context, None)
        .map_err(|error| PolicyError::Request(error.to_string()))
}

/// Compiles one rule's globs, resolving workspace-relative patterns.
fn compile_globs(
    rule: &Rule,
    patterns: &[String],
    workspace_root: &str,
) -> Result<Vec<GlobMatcher>, PolicyError> {
    patterns
        .iter()
        .map(|pattern| {
            Glob::new(&resolve_pattern(pattern, workspace_root))
                .map(|glob| glob.compile_matcher())
                .map_err(|source| PolicyError::Pattern {
                    rule: rule.id.clone(),
                    pattern: pattern.clone(),
                    source,
                })
        })
        .collect()
}

/// Resolves a policy pattern against the workspace root.
///
/// A workspace-relative pattern becomes absolute, so it can never match outside
/// the workspace. A home-relative pattern is left to match anywhere, because home
/// expansion belongs to policy loading ([ADR-0019]).
fn resolve_pattern(pattern: &str, workspace_root: &str) -> String {
    if let Some(rest) = pattern.strip_prefix("./") {
        format!("{}/{rest}", workspace_root.trim_end_matches('/'))
    } else if let Some(rest) = pattern.strip_prefix("~/") {
        format!("**/{rest}")
    } else {
        pattern.to_owned()
    }
}

/// Whether an action type pattern matches an action type. A trailing `*` is a wildcard.
fn action_type_matches(pattern: &str, action_type: &str) -> bool {
    pattern
        .strip_suffix('*')
        .map_or(pattern == action_type, |prefix| {
            action_type.starts_with(prefix)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_relative_patterns_cannot_match_outside_the_workspace() {
        assert_eq!(
            resolve_pattern("./src/**", "/home/dev/project"),
            "/home/dev/project/src/**"
        );
        let glob = Glob::new(&resolve_pattern("./src/**", "/home/dev/project"))
            .unwrap()
            .compile_matcher();
        assert!(glob.is_match("/home/dev/project/src/login.rs"));
        assert!(!glob.is_match("/home/other/src/login.rs"));
    }

    #[test]
    fn action_type_wildcards_match_a_namespace() {
        assert!(action_type_matches("fs.*", "fs.write"));
        assert!(action_type_matches("fs.write", "fs.write"));
        assert!(!action_type_matches("fs.write", "fs.read"));
        assert!(!action_type_matches("fs.*", "net.fetch"));
    }
}
