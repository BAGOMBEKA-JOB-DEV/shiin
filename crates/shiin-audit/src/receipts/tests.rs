//! Offline cryptographic and domain-binding regressions; no keyring is needed.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests use explicit non-production fixtures"
)]

use super::*;
use serde_json::json;

fn payload(key: &SigningKey) -> Value {
    json!({"schema":"shiin.receipt/v1-draft.2", "key_id":key_id(&key.verifying_key()),
        "request_id":"0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b", "decision":"deny", "reason":"deny.policy",
        "policy_snapshot":format!("sha256:{}", "0".repeat(64)), "request_digest":format!("sha256:{}", "1".repeat(64)),
        "signed_at":"2026-10-15T00:00:00Z"})
}

#[test]
fn pae_matches_the_dsse_reference_encoding() {
    assert_eq!(
        pae("http://example.com/HelloWorld", b"hello world"),
        b"DSSEv1 29 http://example.com/HelloWorld 11 hello world"
    );
    assert_ne!(pae("EXAMPLE-1", b"23"), pae("EXAMPLE-12", b"3"));
}

#[test]
fn receipts_verify_offline_and_bind_contents_type_and_trusted_key() {
    let example_key = SigningKey::from_bytes(&[42; 32]);
    let public = example_key.verifying_key().to_bytes();
    let expected = payload(&example_key);
    let envelope = sign(&example_key, RECEIPT_TYPE, &expected).unwrap();
    assert_eq!(verify(&envelope, &public, RECEIPT_TYPE).unwrap(), expected);
    let other = SigningKey::from_bytes(&[43; 32]).verifying_key().to_bytes();
    assert!(verify(&envelope, &other, RECEIPT_TYPE).is_err());
    assert!(verify(&envelope, &public, CHECKPOINT_TYPE).is_err());
    let mut tampered = envelope.clone();
    tampered.payload = STANDARD.encode(b"EXAMPLE-altered");
    assert!(verify(&tampered, &public, RECEIPT_TYPE).is_err());
    tampered = envelope.clone();
    tampered.signatures.clear();
    assert!(verify(&tampered, &public, RECEIPT_TYPE).is_err());
    let mut claims = expected;
    claims["key_id"] = json!("EXAMPLE-other-key");
    assert!(
        verify(
            &sign(&example_key, RECEIPT_TYPE, &claims).unwrap(),
            &public,
            RECEIPT_TYPE
        )
        .is_err()
    );
}

#[test]
fn dsse_accepts_standard_and_url_safe_base64_encodings() {
    let example_key = SigningKey::from_bytes(&[42; 32]);
    let mut envelope = sign(&example_key, RECEIPT_TYPE, &payload(&example_key)).unwrap();
    envelope.payload = URL_SAFE_NO_PAD.encode(decode(&envelope.payload).unwrap());
    envelope.signatures[0].sig =
        URL_SAFE_NO_PAD.encode(decode(&envelope.signatures[0].sig).unwrap());
    assert!(
        verify(
            &envelope,
            &example_key.verifying_key().to_bytes(),
            RECEIPT_TYPE
        )
        .is_ok()
    );
}

#[test]
fn replay_ciphertext_does_not_disclose_input_and_rejects_tampering() {
    let example_key = [17_u8; 32];
    let example_nonce = [18_u8; 24];
    let artifact = encrypt_replay(
        &example_key,
        "EXAMPLE-key",
        "EXAMPLE-request",
        example_nonce,
        b"sk-EXAMPLE-PRIVATE-REPLAY",
    )
    .unwrap();
    assert!(
        !serde_json::to_string(&artifact)
            .unwrap()
            .contains("sk-EXAMPLE-PRIVATE-REPLAY")
    );
    assert_eq!(
        decrypt_replay(&example_key, &artifact).unwrap(),
        b"sk-EXAMPLE-PRIVATE-REPLAY"
    );
    let mut changed = artifact.clone();
    changed.request_id.push_str("-changed");
    assert!(decrypt_replay(&example_key, &changed).is_err());
    changed = artifact.clone();
    changed.ciphertext = STANDARD.encode(b"EXAMPLE-altered");
    assert!(decrypt_replay(&example_key, &changed).is_err());
    assert!(decrypt_replay(&[19_u8; 32], &artifact).is_err());
}
