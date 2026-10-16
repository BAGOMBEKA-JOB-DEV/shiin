//! Reads the operator-facing TOML policy and produces a policy snapshot.
//!
//! Operators write a short file with rules at the top level. That is translated
//! into the layered snapshot model the engine evaluates, so the engine never has to
//! know about the file format ([ADR-0014](../../../docs/adr/0014-cedar-policy-engine-with-toml-front-end.md)).

use serde::Deserialize;
use sha2::{Digest, Sha256};
use shiin_schema::{Capability, DecisionValue, Layer, Policy, RiskSetting, Rule, versions};
use std::collections::BTreeMap;

use crate::PolicyError;

/// The operator-facing form of a policy file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyFile {
    /// Format version of the file. Only `1` is understood.
    version: u32,
    #[serde(default)]
    capabilities: BTreeMap<String, Capability>,
    /// Defaults applied when no rule matches.
    #[serde(default)]
    defaults: Defaults,
    /// The rules, in the order they are written.
    #[serde(default, rename = "rule")]
    rules: Vec<Rule>,
    /// Risk signal configuration.
    #[serde(default)]
    risk: BTreeMap<String, RiskSetting>,
}

/// Defaults section of a policy file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Defaults {
    /// The decision when no rule matches. Never `allow`.
    decision: DecisionValue,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            decision: DecisionValue::Deny,
        }
    }
}

/// Parses an operator-facing policy file into a snapshot.
///
/// The snapshot's digest is the hash of the file's bytes, so a decision can name
/// exactly which policy text produced it.
///
/// # Errors
///
/// Returns an error if the file is not valid TOML, uses an unknown key, declares an
/// unsupported version, or sets a default of `allow`.
pub fn parse(text: &str) -> Result<Policy, PolicyError> {
    let file: PolicyFile = toml::from_str(text).map_err(|error| PolicyError::Syntax {
        message: error.to_string(),
    })?;
    if file.version != 1 {
        return Err(PolicyError::Syntax {
            message: format!("unsupported policy version {}, expected 1", file.version),
        });
    }
    if file.defaults.decision == DecisionValue::Allow {
        return Err(PolicyError::Syntax {
            message: "the default decision may not be `allow`".to_owned(),
        });
    }
    for (signal, setting) in &file.risk {
        if ![
            "opaque_command",
            "secret_path",
            "outside_workspace",
            "bulk_delete",
            "destructive_vcs",
            "privilege_escalation",
        ]
        .contains(&signal.as_str())
        {
            return Err(PolicyError::Syntax {
                message: format!("unknown risk signal `{signal}`"),
            });
        }
        if setting.threshold() == Some(0) {
            return Err(PolicyError::Syntax {
                message: "risk threshold must be positive".to_owned(),
            });
        }
    }

    for (key, capability) in &file.capabilities {
        capability
            .validate(key)
            .map_err(|message| PolicyError::Syntax {
                message: message.to_owned(),
            })?;
    }
    let digest = format!("sha256:{:x}", Sha256::digest(text.as_bytes()));
    Ok(Policy {
        schema: versions::POLICY.to_owned(),
        layers: vec![Layer {
            name: "user".to_owned(),
            digest: digest.clone(),
            rules: file.rules,
        }],
        default: file.defaults.decision,
        capabilities: file.capabilities,
        risk: file.risk,
    })
}

/// The digest of a policy's source text, used as its snapshot identifier.
#[must_use]
pub fn digest(text: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(text.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
version = 1

[defaults]
decision = "deny"

[[rule]]
id = "edit-source"
effect = "allow"
action_types = ["fs.read", "fs.write"]
resources = ["./src/**"]

[[rule]]
id = "no-secrets"
effect = "deny"
action_types = ["fs.read"]
resources = ["**/.env"]
reason = "Secret files are not readable by agents."

[risk]
opaque_command = "pending"
bulk_delete = { decision = "pending", threshold = 20 }
"#;

    #[test]
    fn an_operator_file_becomes_a_snapshot() {
        let policy = parse(EXAMPLE).unwrap();
        assert_eq!(policy.layers.len(), 1);
        assert_eq!(policy.rules().count(), 2);
        assert_eq!(policy.default, DecisionValue::Deny);
        assert_eq!(policy.risk["bulk_delete"].threshold(), Some(20));
        assert!(policy.layers[0].digest.starts_with("sha256:"));
    }

    #[test]
    fn an_unknown_key_is_refused() {
        let text = "version = 1\n[[rule]]\nid = \"x\"\neffect = \"allow\"\nsurprise = true\n";
        assert!(parse(text).is_err());
    }

    #[test]
    fn a_default_of_allow_is_refused() {
        let text = "version = 1\n[defaults]\ndecision = \"allow\"\n";
        assert!(parse(text).is_err());
    }

    #[test]
    fn an_unsupported_version_is_refused() {
        assert!(parse("version = 2\n").is_err());
    }

    #[test]
    fn capabilities_require_an_explicit_unambiguous_grant_or_refusal() {
        for body in [
            "id='EXAMPLE'\naction_types=['shell.exec']",
            "id='EXAMPLE'\nallow=false\naction_types=['shell.exec']",
            "id='EXAMPLE'\nallow=true\ndeny=true\naction_types=['shell.exec']",
            "id='OTHER-EXAMPLE'\nallow=true\naction_types=['shell.exec']",
            "id='EXAMPLE'\nallow=true\naction_types=[]",
            "id='EXAMPLE'\nallow=true\naction_types=['shell.*.exec']",
            "id='EXAMPLE'\nallow=true\naction_types=['shell.exec']\nunknown=true",
        ] {
            assert!(
                parse(&format!("version=1\n[capabilities.EXAMPLE]\n{body}\n")).is_err(),
                "{body}"
            );
        }
        for effect in ["allow", "deny"] {
            assert!(parse(&format!("version=1\n[capabilities.EXAMPLE]\nid='EXAMPLE'\n{effect}=true\naction_types=['shell.exec']\n")).is_ok());
        }
    }
}
