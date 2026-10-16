//! Operator evidence commands. Private keys and plaintext replay inputs are never printed.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use shiin_approval::Store;
use shiin_audit::{
    AuditLog,
    keys::Keys,
    receipts::{self, EncryptedReplay, Envelope},
};
use shiin_engine::{Inputs, decide};
use shiin_policy::CompiledPolicy;
use shiin_schema::{DecisionValue, ReplayInputs};
use std::fs;
use std::io::{IsTerminal, Read};
use std::path::Path;

pub(super) fn key_command(rotate: bool) -> Result<String, String> {
    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        return Err("key administration requires your own operator terminal".to_owned());
    }
    let keys = if rotate {
        Keys::rotate()?
    } else {
        Keys::initialize()?
    };
    Ok(format!("active signing key: {}", keys.id()))
}

pub(super) fn key_public() -> Result<String, String> {
    let keys = Keys::active()?;
    serde_json::to_string_pretty(
        &json!({"key_id":keys.id(), "public_key":STANDARD.encode(keys.public())}),
    )
    .map_err(|error| error.to_string())
}

fn read_json(path: &str) -> Result<Value, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|error| error.to_string())?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("evidence file exceeds limit".to_owned());
    }
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

pub(super) fn verify_evidence(artifact: &str, public: &str) -> Result<String, String> {
    let envelope: Envelope =
        serde_json::from_value(read_json(artifact)?).map_err(|error| error.to_string())?;
    let public = read_json(public)?;
    let trusted: [u8; 32] =
        receipts::decode(public["public_key"].as_str().ok_or("public key missing")?)?
            .try_into()
            .map_err(|_| "invalid public key length")?;
    let claims = receipts::verify(&envelope, &trusted, &envelope.payload_type)?;
    if envelope.payload_type == receipts::CHECKPOINT_TYPE {
        let log = AuditLog::new(super::AUDIT_PATH);
        if !log.verify().map_err(|error| error.to_string())?.is_empty() {
            return Err("audit chain is broken".to_owned());
        }
        let events = log.read().map_err(|error| error.to_string())?;
        let seq = claims["seq"]
            .as_u64()
            .ok_or("checkpoint position missing")?;
        let seq =
            usize::try_from(seq).map_err(|_| "checkpoint position exceeds platform limits")?;
        let head = if seq == 0 {
            Some(shiin_audit::GENESIS)
        } else {
            events.get(seq - 1).map(|event| event.hash.as_str())
        };
        if head != claims["head"].as_str() {
            return Err("audit history does not match signed checkpoint".to_owned());
        }
    }
    Ok(format!(
        "verified {} signed by {}",
        envelope.payload_type,
        claims["key_id"].as_str().ok_or("key id missing")?
    ))
}

pub(super) fn replay_evidence(
    artifact: &str,
    alternate_policy: Option<&str>,
) -> Result<String, String> {
    let envelope: Envelope =
        serde_json::from_value(read_json(artifact)?).map_err(|error| error.to_string())?;
    if envelope.payload_type != receipts::REPLAY_TYPE {
        return Err("expected an encrypted replay envelope".to_owned());
    }
    // The identifier selects a candidate key; the signature authenticates it before decryption.
    let encrypted: EncryptedReplay = serde_json::from_slice(&receipts::decode(&envelope.payload)?)
        .map_err(|error| error.to_string())?;
    let keys = Keys::load(&encrypted.key_id)?;
    let verified = receipts::verify(&envelope, &keys.public(), receipts::REPLAY_TYPE)?;
    let encrypted: EncryptedReplay =
        serde_json::from_value(verified).map_err(|error| error.to_string())?;
    let record: ReplayInputs<Store> =
        serde_json::from_slice(&keys.decrypt(&encrypted)?).map_err(|error| error.to_string())?;
    replay_record(record, &encrypted.request_id, alternate_policy)
}

fn replay_record(
    record: ReplayInputs<Store>,
    encrypted_request_id: &str,
    alternate_policy: Option<&str>,
) -> Result<String, String> {
    if record.schema != "shiin.replay-input/v1-draft.1"
        || record.engine_version != shiin_engine::ENGINE_VERSION
        || record.request.id != encrypted_request_id
    {
        return Err("unsupported or mismatched replay inputs".to_owned());
    }
    record.request.validate().map_err(str::to_owned)?;
    let mut request = record.request;
    let source = if let Some(path) = alternate_policy {
        let (source, _) = super::load_policy(Path::new(path), &request.workspace.root)?;
        request
            .policy_snapshot
            .clone_from(&source.layers.first().ok_or("snapshot missing")?.digest);
        source
    } else {
        record.policy
    };
    let compiled = CompiledPolicy::compile(&source, &request.workspace.root)
        .map_err(|error| error.to_string())?;
    let mut decision = decide(&Inputs {
        request: &request,
        policy: &compiled,
        source: &source,
        task: record.task.as_ref(),
        usage: record.usage,
        now: &record.now,
        mode: record.mode,
    });
    if decision.decision == DecisionValue::Pending
        && record
            .grants
            .covered(&request, shiin_approval::deadline(&record.now)?)?
    {
        decision.decision = DecisionValue::Allow;
        "allow.grant".clone_into(&mut decision.reason);
    }
    let matched =
        decision.decision == record.decision.decision && decision.reason == record.decision.reason;
    if alternate_policy.is_none() && !matched {
        return Err("replay did not reproduce the recorded decision and reason".to_owned());
    }
    Ok(format!(
        "replay: {:?} ({}) -> {:?} ({}){}",
        record.decision.decision,
        record.decision.reason,
        decision.decision,
        decision.reason,
        if matched {
            " [unchanged]"
        } else {
            " [changed]"
        }
    ))
}

#[cfg(test)]
#[path = "evidence/tests.rs"]
mod tests;
