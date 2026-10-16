//! The full request sent to the Mélodium API, as the API is expected to receive it.
//!
//! `fixtures/request.json` is shared with the API tests for `ExecutionRunRequest`,
//! so the two hand-written copies of the contract cannot drift apart unnoticed.

use work_mel::api::{Arch, ModeRequest, Request, System};

const FIXTURE: &str = include_str!("fixtures/request.json");

#[test]
fn fixture_deserializes() {
    let request: Request = serde_json::from_str(FIXTURE).unwrap();

    assert_eq!(request.edition, None);
    assert_eq!(request.arch, Some(Arch::Arm64));
    assert!(matches!(request.system.0, Some(System::Os(_))));
    assert!(matches!(
        request.mode,
        ModeRequest::DistributionSecretKey { .. }
    ));
    assert_eq!(request.tags, vec!["ci", "nightly"]);
    assert_eq!(request.containers[0].arch, Some(Arch::Arm64));
    assert_eq!(request.service_containers[0].arch, None);
}

#[test]
fn fixture_roundtrips() {
    let request: Request = serde_json::from_str(FIXTURE).unwrap();
    let expected: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();

    assert_eq!(serde_json::to_value(&request).unwrap(), expected);
}
