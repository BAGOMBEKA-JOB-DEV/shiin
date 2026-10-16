//! Replay the published corpus through signed, encrypted evidence without an OS keyring.

use super::replay_record;
use ed25519_dalek::SigningKey;
use serde_json::Value;
use shiin_approval::Store;
use shiin_audit::{canonical_bytes, receipts};
use shiin_engine::{Inputs, decide};
use shiin_policy::CompiledPolicy;
use shiin_schema::{ActionRequest, Mode, Policy, ReplayInputs, TaskDeclaration, TaskUsage};
use std::fs;
use std::path::Path;

fn record(case: &Value) -> Result<ReplayInputs<Store>, Box<dyn std::error::Error>> {
    let request: ActionRequest = serde_json::from_value(case["action_request"].clone())?;
    let policy: Policy = serde_json::from_value(case["policy"].clone())?;
    let task: Option<TaskDeclaration> = case
        .get("task")
        .filter(|value| value.is_object())
        .cloned()
        .map(serde_json::from_value)
        .transpose()?;
    let usage: TaskUsage = case
        .get("usage")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let now = case["now"]
        .as_str()
        .unwrap_or("1970-01-01T00:00:00Z")
        .to_owned();
    let mode = if case["mode"].as_str() == Some("unattended") {
        Mode::Unattended
    } else {
        Mode::Attended
    };
    let compiled = CompiledPolicy::compile(&policy, &request.workspace.root)?;
    let decision = decide(&Inputs {
        request: &request,
        policy: &compiled,
        source: &policy,
        task: task.as_ref(),
        usage,
        now: &now,
        mode,
    });
    assert_eq!(
        serde_json::to_value(decision.decision)?,
        case["expected"]["decision"]
    );
    assert_eq!(decision.reason, case["expected"]["reason"]);
    Ok(ReplayInputs {
        schema: "shiin.replay-input/v1-draft.1".to_owned(),
        request,
        policy,
        task,
        usage,
        now,
        mode,
        decision,
        grants: Store::default(),
        engine_version: shiin_engine::ENGINE_VERSION.to_owned(),
    })
}

#[test]
fn signed_encrypted_replay_reproduces_every_conformance_case()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/test-vectors");
    let example_signing = SigningKey::from_bytes(&[42; 32]);
    let example_encryption = [17_u8; 32];
    let key_id = receipts::key_id(&example_signing.verifying_key());
    let mut count = 0_u64;
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let case: Value = serde_json::from_slice(&fs::read(path)?)?;
        let record = record(&case)?;
        let mut nonce = [0_u8; 24];
        nonce[..8].copy_from_slice(&count.to_le_bytes());
        let encrypted = receipts::encrypt_replay(
            &example_encryption,
            &key_id,
            &record.request.id,
            nonce,
            &canonical_bytes(&serde_json::to_value(&record)?)?,
        )?;
        let envelope = receipts::sign(
            &example_signing,
            receipts::REPLAY_TYPE,
            &serde_json::to_value(&encrypted)?,
        )?;
        let verified = receipts::verify(
            &envelope,
            &example_signing.verifying_key().to_bytes(),
            receipts::REPLAY_TYPE,
        )?;
        let encrypted = serde_json::from_value(verified)?;
        let recovered: ReplayInputs<Store> =
            serde_json::from_slice(&receipts::decrypt_replay(&example_encryption, &encrypted)?)?;
        let result = replay_record(recovered, &encrypted.request_id, None)?;
        assert!(result.ends_with("[unchanged]"), "{}: {result}", case["id"]);
        count += 1;
    }
    assert!(count >= 26, "the entire published corpus is replayed");
    Ok(())
}

#[test]
fn replay_refuses_wrong_bindings_versions_and_recorded_results()
-> Result<(), Box<dyn std::error::Error>> {
    let case: Value = serde_json::from_str(include_str!(
        "../../../../docs/spec/test-vectors/tv-001-allowed-edit.v1.json"
    ))?;
    let original = record(&case)?;
    assert!(replay_record(original.clone(), "EXAMPLE-different-request", None).is_err());
    let mut changed = original.clone();
    changed.engine_version = "EXAMPLE-unsupported-engine".to_owned();
    assert!(replay_record(changed, &original.request.id, None).is_err());
    let mut changed = original.clone();
    changed.decision.decision = shiin_schema::DecisionValue::Deny;
    assert!(replay_record(changed, &original.request.id, None).is_err());
    let mut changed = original.clone();
    changed.decision.reason = "EXAMPLE-incorrect-reason".to_owned();
    assert!(replay_record(changed, &original.request.id, None).is_err());
    Ok(())
}

#[test]
fn replay_retains_grant_coverage_but_cannot_override_a_deny()
-> Result<(), Box<dyn std::error::Error>> {
    let case: Value = serde_json::from_str(include_str!(
        "../../../../docs/spec/test-vectors/tv-003-opaque-command.v1.json"
    ))?;
    let mut recorded = record(&case)?;
    let now = shiin_approval::deadline(&recorded.now)?;
    let mut grants = Store::default();
    grants.apply(&recorded.request, &mut recorded.decision, now, None)?;
    let ticket = recorded
        .decision
        .approval
        .as_ref()
        .ok_or("pending ticket missing")?
        .ticket
        .clone();
    grants.resolve(&ticket, shiin_schema::Resolution::AllowOnce, None, 60, now)?;
    recorded.grants = grants.clone();
    grants.apply(&recorded.request, &mut recorded.decision, now, None)?;
    let allowed = replay_record(recorded.clone(), &recorded.request.id, None)?;
    assert!(allowed.contains("allow.grant"));
    assert!(allowed.ends_with("[unchanged]"));

    recorded.policy.risk.insert(
        "opaque_command".to_owned(),
        serde_json::from_value(serde_json::json!("deny"))?,
    );
    recorded.decision.decision = shiin_schema::DecisionValue::Deny;
    recorded.decision.reason = "deny.risk".to_owned();
    let denied = replay_record(recorded.clone(), &recorded.request.id, None)?;
    assert!(denied.contains("deny.risk"));
    assert!(denied.ends_with("[unchanged]"));
    Ok(())
}

#[test]
fn what_if_replay_compares_a_different_policy() -> Result<(), Box<dyn std::error::Error>> {
    let case: Value = serde_json::from_str(include_str!(
        "../../../../docs/spec/test-vectors/tv-001-allowed-edit.v1.json"
    ))?;
    let original = record(&case)?;
    let path = std::env::temp_dir().join(format!(
        "shiin-replay-{}.toml",
        shiin_schema::clock::new_id()
    ));
    fs::write(
        &path,
        "version = 1\n[capabilities.EXAMPLE]\nid = 'EXAMPLE'\nallow = true\naction_types = ['*']\n[defaults]\ndecision = 'deny'\n",
    )?;
    let result = replay_record(original.clone(), &original.request.id, path.to_str());
    fs::remove_file(path)?;
    let result = result?;
    assert!(result.contains("allow.policy"));
    assert!(result.contains("deny.default"));
    assert!(result.ends_with("[changed]"));
    Ok(())
}
