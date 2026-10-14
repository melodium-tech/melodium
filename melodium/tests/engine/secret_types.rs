//! `Secret<T>` is usable as a port type, implements no trait but `Serialize` and
//! `Deserialize` (following its inner type), and cannot hold another secret.

use melodium::{load_raw, LoadingConfig};
use std::sync::Arc;

fn load(script: &str) -> Result<(), String> {
    let result = load_raw(
        Arc::new(script.as_bytes().to_vec()),
        "main",
        LoadingConfig {
            core_packages: Vec::new(),
            search_locations: Vec::new(),
            raw_elements: Vec::new(),
        },
    );
    let errors = result
        .errors()
        .iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    match result.into_result() {
        Ok(_) if errors.is_empty() => Ok(()),
        Ok(_) => Err(errors),
        Err(failure) => Err(format!("{failure}\n{errors}")),
    }
}

#[test]
fn secrets_flow_through_ports() {
    load(include_str!("scripts/secret_ports.mel")).unwrap();
}

#[test]
fn secrets_do_not_satisfy_trait_bounds() {
    let errors = load(include_str!("scripts/secret_trait_bound.mel")).unwrap_err();
    assert!(
        errors.contains("Type 'Secret<string>'")
            && errors.contains("does not satisfy trait ToString"),
        "{}",
        errors
    );
}

#[test]
fn secrets_can_be_given_to_generic_distribution_treatments() {
    load(include_str!("scripts/secret_send.mel")).unwrap();
}

#[test]
fn secrets_cannot_be_nested() {
    let errors = load(include_str!("scripts/secret_nested.mel")).unwrap_err();
    assert!(
        errors.contains("S0189") && errors.contains("'Secret' is not a valid type"),
        "{}",
        errors
    );
}
