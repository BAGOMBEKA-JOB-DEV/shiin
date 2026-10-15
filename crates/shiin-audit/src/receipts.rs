//! DSSE envelopes, Ed25519 verification, and authenticated encryption for replay inputs.
//!
//! DSSE protocol v1.0.2: <https://github.com/secure-systems-lab/dsse/blob/master/protocol.md>
//! (verified 2026-10-15). Verification consumes exactly the bytes whose signature was checked.

use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD},
};
use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305,
    aead::{Aead, Payload},
};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Media type of a decision receipt.
pub const RECEIPT_TYPE: &str = "application/vnd.shiin.receipt.v1+json";
/// Media type of an audit checkpoint.
pub const CHECKPOINT_TYPE: &str = "application/vnd.shiin.checkpoint.v1+json";
/// Media type of encrypted, signed replay inputs.
pub const REPLAY_TYPE: &str = "application/vnd.shiin.replay.v1+json";
const MAX_BYTES: usize = 8 * 1024 * 1024;

#[cfg(test)]
mod tests;

/// One DSSE signature. Its key identifier is only a lookup hint.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeSignature {
    /// Identifier of the public key used to produce this signature.
    pub keyid: String,
    /// Base64-encoded Ed25519 signature.
    pub sig: String,
}

/// The standard DSSE JSON envelope.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    /// Authenticated payload interpretation.
    #[serde(rename = "payloadType")]
    pub payload_type: String,
    /// Base64-encoded serialized payload bytes.
    pub payload: String,
    /// Signatures over DSSE pre-authentication encoding.
    pub signatures: Vec<EnvelopeSignature>,
}

/// Authenticated ciphertext; normalized commands and policy inputs remain encrypted at rest.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EncryptedReplay {
    /// Wire format version.
    pub schema: String,
    /// OS-keyring entry containing the replay encryption key.
    pub key_id: String,
    /// Correlation identifier, also bound as AEAD associated data.
    pub request_id: String,
    /// Random 192-bit XChaCha nonce.
    pub nonce: String,
    /// Authenticated ciphertext, encoded as base64.
    pub ciphertext: String,
}

/// Fingerprint of a trusted public key.
#[must_use]
pub fn key_id(key: &VerifyingKey) -> String {
    format!("sha256:{:x}", Sha256::digest(key.as_bytes()))
}

/// DSSE's unambiguous, length-prefixed pre-authentication encoding.
#[must_use]
pub fn pae(payload_type: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes = format!(
        "DSSEv1 {} {payload_type} {} ",
        payload_type.len(),
        body.len()
    )
    .into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

/// Signs serialized JSON without embedding private material.
///
/// # Errors
/// Unsupported types, oversized payloads, and serialization failures are rejected.
pub fn sign(key: &SigningKey, payload_type: &str, payload: &Value) -> Result<Envelope, String> {
    if ![RECEIPT_TYPE, CHECKPOINT_TYPE, REPLAY_TYPE].contains(&payload_type) {
        return Err("unsupported evidence payload type".to_owned());
    }
    let body = crate::canonical_bytes(payload).map_err(|error| error.to_string())?;
    if body.len() > MAX_BYTES {
        return Err("evidence payload exceeds limit".to_owned());
    }
    let signature = key.sign(&pae(payload_type, &body));
    Ok(Envelope {
        payload_type: payload_type.to_owned(),
        payload: STANDARD.encode(body),
        signatures: vec![EnvelopeSignature {
            keyid: key_id(&key.verifying_key()),
            sig: STANDARD.encode(signature.to_bytes()),
        }],
    })
}

/// Verifies offline against a caller-supplied trusted public key and interpretation.
///
/// # Errors
/// Any mismatch, weak key, malformed encoding, missing signature, or invalid payload fails closed.
pub fn verify(
    envelope: &Envelope,
    trusted_key: &[u8; 32],
    expected_type: &str,
) -> Result<Value, String> {
    if envelope.payload_type != expected_type
        || ![RECEIPT_TYPE, CHECKPOINT_TYPE, REPLAY_TYPE].contains(&expected_type)
        || envelope.signatures.is_empty()
        || envelope.signatures.len() > 16
        || envelope.payload.len() > MAX_BYTES * 2
    {
        return Err("unsupported or oversized evidence envelope".to_owned());
    }
    let key = VerifyingKey::from_bytes(trusted_key).map_err(|_| "invalid trusted public key")?;
    if key.is_weak() {
        return Err("weak trusted public key".to_owned());
    }
    let body = decode(&envelope.payload)?;
    if body.len() > MAX_BYTES {
        return Err("evidence payload exceeds limit".to_owned());
    }
    let authenticated = pae(expected_type, &body);
    let valid = envelope.signatures.iter().any(|item| {
        decode(&item.sig)
            .ok()
            .and_then(|bytes| Signature::try_from(bytes.as_slice()).ok())
            .is_some_and(|signature| key.verify_strict(&authenticated, &signature).is_ok())
    });
    if !valid {
        return Err("evidence signature does not verify".to_owned());
    }
    let value: Value =
        serde_json::from_slice(&body).map_err(|_| "verified evidence is not valid JSON")?;
    if value.get("key_id").and_then(Value::as_str) != Some(key_id(&key).as_str()) {
        return Err("signed key identifier does not match trusted key".to_owned());
    }
    validate_payload(&value, expected_type)?;
    Ok(value)
}

fn validate_payload(value: &Value, kind: &str) -> Result<(), String> {
    let schema = match kind {
        RECEIPT_TYPE => "shiin.receipt/v1-draft.2",
        CHECKPOINT_TYPE => "shiin.checkpoint/v1-draft.1",
        _ => "shiin.replay-encrypted/v1-draft.1",
    };
    if value.get("schema").and_then(Value::as_str) != Some(schema) {
        return Err("unsupported signed payload schema".to_owned());
    }
    if kind == RECEIPT_TYPE {
        if !matches!(
            value["decision"].as_str(),
            Some("allow" | "deny" | "pending")
        ) || value["request_id"].as_str().is_none_or(str::is_empty)
            || value["reason"].as_str().is_none_or(str::is_empty)
            || !is_digest(&value["policy_snapshot"])
            || !is_digest(&value["request_digest"])
            || value["signed_at"].as_str().is_none_or(str::is_empty)
        {
            return Err("incomplete receipt claims".to_owned());
        }
    } else if kind == CHECKPOINT_TYPE
        && (value["seq"].as_u64().is_none() || !is_digest(&value["head"]))
    {
        return Err("incomplete checkpoint claims".to_owned());
    }
    Ok(())
}

fn is_digest(value: &Value) -> bool {
    value
        .as_str()
        .and_then(|value| value.strip_prefix("sha256:"))
        .is_some_and(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
}

/// Accepts both DSSE base64 alphabets, with or without padding.
///
/// # Errors
/// Invalid encoding never becomes an empty payload.
pub fn decode(value: &str) -> Result<Vec<u8>, String> {
    STANDARD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .or_else(|_| STANDARD_NO_PAD.decode(value))
        .or_else(|_| URL_SAFE_NO_PAD.decode(value))
        .map_err(|_| "invalid evidence base64".to_owned())
}

/// Encrypts complete replay inputs using a caller-generated fresh nonce and OS-keyring key.
///
/// # Errors
/// Oversized payloads and encryption failures prevent evidence issuance.
pub fn encrypt_replay(
    key: &[u8; 32],
    id: &str,
    request_id: &str,
    nonce: [u8; 24],
    plaintext: &[u8],
) -> Result<EncryptedReplay, String> {
    if plaintext.len() > MAX_BYTES {
        return Err("replay inputs exceed limit".to_owned());
    }
    let cipher = XChaCha20Poly1305::new(key.into());
    let aad = format!("shiin.replay/v1:{id}:{request_id}");
    let ciphertext = cipher
        .encrypt(
            (&nonce).into(),
            Payload {
                msg: plaintext,
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| "replay encryption failed")?;
    Ok(EncryptedReplay {
        schema: "shiin.replay-encrypted/v1-draft.1".to_owned(),
        key_id: id.to_owned(),
        request_id: request_id.to_owned(),
        nonce: STANDARD.encode(nonce),
        ciphertext: STANDARD.encode(ciphertext),
    })
}

/// Decrypts only authenticated replay bytes. Verify the signed envelope before calling.
///
/// # Errors
/// Unknown schema, invalid nonce, altered ciphertext, or mismatched binding is rejected.
pub fn decrypt_replay(key: &[u8; 32], artifact: &EncryptedReplay) -> Result<Vec<u8>, String> {
    if artifact.schema != "shiin.replay-encrypted/v1-draft.1"
        || artifact.ciphertext.len() > MAX_BYTES * 2
    {
        return Err("unsupported replay artifact".to_owned());
    }
    let nonce: [u8; 24] = decode(&artifact.nonce)?
        .try_into()
        .map_err(|_| "invalid replay nonce")?;
    let ciphertext = decode(&artifact.ciphertext)?;
    let aad = format!(
        "shiin.replay/v1:{}:{}",
        artifact.key_id, artifact.request_id
    );
    XChaCha20Poly1305::new(key.into())
        .decrypt(
            (&nonce).into(),
            Payload {
                msg: &ciphertext,
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| "replay authentication failed".to_owned())
}
