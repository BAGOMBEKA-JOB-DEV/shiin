//! Verifies that intercepted callers cannot claim operator authority.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests construct controlled temporary state"
)]

use serde_json::json;
use shiin_schema::clock;
use shiind::{OperatorKey, Service};
use std::fs;

#[test]
fn intercepted_requests_cannot_resolve_revoke_or_read_operator_state() {
    let directory = std::env::temp_dir().join(format!("shiin-authority-{}", clock::new_id()));
    fs::create_dir_all(&directory).unwrap();
    let key = OperatorKey::generate().unwrap();
    let proof = key.for_operator_terminal().to_owned();
    let mut service = Service::open(&directory, key).unwrap();
    for method in [
        "shiin.pending",
        "shiin.grants",
        "shiin.approve.resolve",
        "shiin.revoke",
    ] {
        for params in [
            json!({}),
            json!({"role":"operator"}),
            json!({"proof":"EXAMPLE-invalid", "ticket":"EXAMPLE-ticket", "resolution":"allow_once", "lifetime":60}),
        ] {
            assert!(
                service.dispatch(method, params).is_err(),
                "an intercepted caller claimed authority for {method}"
            );
        }
    }
    assert_eq!(
        service
            .dispatch("shiin.pending", json!({"proof":proof}))
            .unwrap(),
        json!([])
    );
    drop(service);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn corrupted_durable_state_is_never_reset() {
    let directory = std::env::temp_dir().join(format!("shiin-corrupt-{}", clock::new_id()));
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("approvals.json"), "{}").unwrap();
    assert!(Service::open(&directory, OperatorKey::generate().unwrap()).is_err());
    assert_eq!(
        fs::read_to_string(directory.join("approvals.json")).unwrap(),
        "{}"
    );
    fs::remove_dir_all(directory).unwrap();
}
