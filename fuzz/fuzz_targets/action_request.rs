//! Parses and validates untrusted wire records and checks validated round trips.
#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use shiin_schema::ActionRequest;

fuzz_target!(|data: &[u8]| {
    if let Ok(action) = serde_json::from_slice::<ActionRequest>(data) {
        if action.validate().is_ok() {
            let encoded = serde_json::to_vec(&action).expect("wire record serializes");
            let round_trip: ActionRequest =
                serde_json::from_slice(&encoded).expect("wire record parses");
            assert!(round_trip.validate().is_ok());
            assert_eq!(
                serde_json::to_value(&action).expect("wire value"),
                serde_json::to_value(&round_trip).expect("round-trip value")
            );
        }
    }
});
