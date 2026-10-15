//! State-machine invariants, including bounded exhaustive operation sequences.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests use self-contained valid fixtures"
)]

use super::*;
use shiin_schema::versions;

fn fixture() -> (Store, ActionRequest, Decision, String) {
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/spec/test-vectors/tv-001-allowed-edit.v1.json"
    ))
    .unwrap();
    let request: ActionRequest = serde_json::from_value(value["action_request"].clone()).unwrap();
    let mut decision = held(&request);
    let mut store = Store::default();
    store.apply(&request, &mut decision, 100, None).unwrap();
    let ticket = decision.approval.as_ref().unwrap().ticket.clone();
    (store, request, decision, ticket)
}

fn held(request: &ActionRequest) -> Decision {
    Decision {
        schema: versions::DECISION.to_owned(),
        request_id: request.id.clone(),
        decision: DecisionValue::Pending,
        reason: "pending.grantable".to_owned(),
        message: DecisionValue::Pending.message().to_owned(),
        approval: None,
        trace: None,
        stage: "policy".to_owned(),
        matched_rules: Vec::new(),
        policy_snapshot: request.policy_snapshot.clone(),
        engine_version: "EXAMPLE".to_owned(),
    }
}

#[test]
fn correlation_ids_change_but_every_semantic_field_is_bound() {
    let (_, request, _, _) = fixture();
    let digest = binding(&request).unwrap();
    let mut retry = request.clone();
    retry.id = clock::new_id();
    assert_eq!(binding(&retry).unwrap(), digest);
    retry.effects[0].value.push_str("-changed");
    assert_ne!(binding(&retry).unwrap(), digest);
    retry = request.clone();
    retry.session.push_str("-other");
    assert_ne!(binding(&retry).unwrap(), digest);
    retry = request.clone();
    retry.policy_snapshot = format!("sha256:{}", "1".repeat(64));
    assert_ne!(binding(&retry).unwrap(), digest);
}

#[test]
fn a_single_use_grant_cannot_be_replayed_or_override_a_denial() {
    let (mut store, mut request, _, ticket) = fixture();
    store
        .resolve(&ticket, Resolution::AllowOnce, None, 60, 101)
        .unwrap();
    let mut denied = held(&request);
    denied.decision = DecisionValue::Deny;
    store.apply(&request, &mut denied, 102, None).unwrap();
    assert_eq!(denied.decision, DecisionValue::Deny);
    request.id = clock::new_id();
    let mut first = held(&request);
    store.apply(&request, &mut first, 102, None).unwrap();
    assert_eq!(first.decision, DecisionValue::Allow);
    let mut second = held(&request);
    store.apply(&request, &mut second, 103, None).unwrap();
    assert_eq!(second.decision, DecisionValue::Pending);
}

#[test]
fn expiry_revocation_and_task_deadlines_are_enforced() {
    let (mut store, _request, _, ticket) = fixture();
    assert!(
        store
            .resolve(&ticket, Resolution::AllowOnce, None, 60, 400)
            .is_err()
    );
    store.revoke(&ticket).unwrap();
    assert!(
        store
            .resolve(&ticket, Resolution::AllowOnce, None, 60, 101)
            .is_err()
    );
    let (mut store, request, _, ticket) = fixture();
    store
        .resolve(&ticket, Resolution::AllowOnce, None, 1, 101)
        .unwrap();
    let mut decision = held(&request);
    store.apply(&request, &mut decision, 102, None).unwrap();
    assert_ne!(decision.decision, DecisionValue::Allow);
    let mut store = Store::default();
    assert!(
        store
            .apply(&request, &mut held(&request), 100, Some(100))
            .is_err()
    );
}

#[test]
fn reusable_scopes_cover_every_effect_and_remain_snapshot_bound() {
    let (mut store, request, _, ticket) = fixture();
    let scope = GrantScope {
        agent: request.agent.id.clone(),
        action_types: vec![request.action_type.clone()],
        workspace: request.workspace.root.clone(),
        resource: format!("{}/src/**", request.workspace.root),
    };
    store
        .resolve(&ticket, Resolution::AllowAlways, Some(scope), 60, 101)
        .unwrap();
    for _ in 0..2 {
        let mut decision = held(&request);
        store.apply(&request, &mut decision, 102, None).unwrap();
        assert_eq!(decision.decision, DecisionValue::Allow);
    }
    let mut other = request.clone();
    other.effects[0].value = "/EXAMPLE-outside/file".to_owned();
    let mut decision = held(&other);
    store.apply(&other, &mut decision, 102, None).unwrap();
    assert_ne!(decision.decision, DecisionValue::Allow);
    other = request.clone();
    other.policy_snapshot = format!("sha256:{}", "1".repeat(64));
    let mut decision = held(&other);
    store.apply(&other, &mut decision, 102, None).unwrap();
    assert_ne!(decision.decision, DecisionValue::Allow);
}

#[test]
fn exhaustive_transition_sequences_never_spend_one_grant_twice() {
    // All 5^5 sequences: resolve, retry, revoke, reload, advance beyond expiry.
    for sequence in 0_u32..3125 {
        let (mut store, request, _, ticket) = fixture();
        let mut operations = sequence;
        let mut allows = 0;
        let mut now = 101;
        for _ in 0..5 {
            match operations % 5 {
                0 => {
                    let _ = store.resolve(&ticket, Resolution::AllowOnce, None, 60, now);
                }
                1 => {
                    let mut decision = held(&request);
                    store.apply(&request, &mut decision, now, None).unwrap();
                    allows += usize::from(decision.decision == DecisionValue::Allow);
                }
                2 => {
                    if let Some(id) = store.grants.first().map(|grant| grant.id.clone()) {
                        store.revoke(&id).unwrap();
                    }
                }
                3 => {
                    store = serde_json::from_slice(&serde_json::to_vec(&store).unwrap()).unwrap();
                }
                _ => now = 500,
            }
            assert!(
                allows <= 1,
                "sequence {sequence} consumed one authorization twice"
            );
            store.validate().unwrap();
            operations /= 5;
        }
    }
}
