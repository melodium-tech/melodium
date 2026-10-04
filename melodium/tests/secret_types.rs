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
    load(
        r#"#!/usr/bin/env melodium
#! name = secret_ports
#! version = 0.11.0
#! require = std:0.11.0

treatment forward()
  input secret: Block<Secret<string>>
  input secrets: Stream<Vec<Secret<Vec<byte>>>>
  output forwarded: Block<Secret<string>>
  output forwarded_all: Stream<Vec<Secret<Vec<byte>>>>
{
    Self.secret -> Self.forwarded
    Self.secrets -> Self.forwarded_all
}

treatment main()
{
}
"#,
    )
    .unwrap();
}

#[test]
fn secrets_do_not_satisfy_trait_bounds() {
    let errors = load(
        r#"#!/usr/bin/env melodium
#! name = secret_trait_bound
#! version = 0.11.0
#! require = std:0.11.0

use std/conv::toString

treatment leak()
  input secret: Stream<Secret<string>>
  output text: Stream<string>
{
    toString<Secret<string>>()

    Self.secret -> toString.value,into -> Self.text
}

treatment main()
{
}
"#,
    )
    .unwrap_err();
    assert!(
        errors.contains("Type 'Secret<string>'")
            && errors.contains("does not satisfy trait ToString"),
        "{}",
        errors
    );
}

#[test]
fn secrets_can_be_given_to_generic_distribution_treatments() {
    load(
        r#"#!/usr/bin/env melodium
#! name = secret_send
#! version = 0.11.0
#! require = std:0.11.0
#! require = distrib:0.11.0

use distrib::DistributionEngine
use distrib::sendBlock

treatment send[distributor: DistributionEngine]()
  input distribution_id: Block<u64>
  input secret: Block<Secret<string>>
{
    sendBlock<Secret<string>>[distributor=distributor](name="secret")

    Self.distribution_id -> sendBlock.distribution_id
    Self.secret -> sendBlock.data
}

treatment main()
{
}
"#,
    )
    .unwrap();
}

#[test]
fn secrets_cannot_be_nested() {
    let errors = load(
        r#"#!/usr/bin/env melodium
#! name = secret_nested
#! version = 0.11.0
#! require = std:0.11.0

treatment forward()
  input secret: Block<Vec<Secret<Option<Secret<string>>>>>
{
}

treatment main()
{
}
"#,
    )
    .unwrap_err();
    assert!(
        errors.contains("S0189") && errors.contains("'Secret' is not a valid type"),
        "{}",
        errors
    );
}
