//! Checks authorization invariants with arbitrary validated action requests.
#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use shiin_engine::{Inputs, decide};
use shiin_policy::{CompiledPolicy, toml_front_end};
use shiin_schema::{ActionRequest, Capability, DecisionValue, Mode, Policy, TaskUsage};
use std::sync::LazyLock;

const ROOT: &str = "/shiin-EXAMPLE-fuzz-root";

struct Fixture {
    source: Policy,
    compiled: CompiledPolicy,
    missing_capabilities: Policy,
    refused_capabilities: Policy,
}

static FIXTURE: LazyLock<Fixture> = LazyLock::new(|| {
    let source = toml_front_end::parse(
        "version=1\n[capabilities.EXAMPLE]\nid='EXAMPLE'\nallow=true\naction_types=['*']\n\
         [[rule]]\nid='EXAMPLE-file-actions'\neffect='allow'\naction_types=['fs.*']\n\
         [[rule]]\nid='EXAMPLE-shell-actions'\neffect='allow'\naction_types=['shell.exec']\n\
         [risk]\nopaque_command='pending'\nsecret_path='deny'\n",
    )
    .expect("constant fuzz policy is valid");
    let compiled = CompiledPolicy::compile(&source, ROOT).expect("constant policy compiles");
    let mut missing_capabilities = source.clone();
    missing_capabilities.capabilities.clear();
    let mut refused_capabilities = source.clone();
    refused_capabilities.capabilities.insert(
        "EXAMPLE-refuse".to_owned(),
        Capability {
            id: "EXAMPLE-refuse".to_owned(),
            allow: None,
            deny: Some(true),
            action_types: vec!["*".to_owned()],
            agents: Vec::new(),
        },
    );
    Fixture {
        source,
        compiled,
        missing_capabilities,
        refused_capabilities,
    }
});

fuzz_target!(|bytes: &[u8]| {
    let Ok(request) = serde_json::from_slice::<ActionRequest>(bytes) else {
        return;
    };
    if request.validate().is_err() {
        return;
    }
    let fixture = &*FIXTURE;
    let mut inputs = Inputs {
        request: &request,
        policy: &fixture.compiled,
        source: &fixture.source,
        task: None,
        usage: TaskUsage::default(),
        now: "2026-10-16T00:00:00Z",
        mode: Mode::Attended,
    };
    let first = decide(&inputs);
    let second = decide(&inputs);
    assert_eq!(
        serde_json::to_value(&first).expect("decision serialization"),
        serde_json::to_value(&second).expect("decision serialization")
    );
    assert_eq!(first.request_id, request.id);
    inputs.mode = Mode::Unattended;
    assert_ne!(decide(&inputs).decision, DecisionValue::Pending);
    inputs.source = &fixture.missing_capabilities;
    assert_eq!(decide(&inputs).decision, DecisionValue::Deny);
    inputs.source = &fixture.refused_capabilities;
    assert_eq!(decide(&inputs).decision, DecisionValue::Deny);
});
