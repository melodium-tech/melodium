//! `Option<T>` parameters declared in Rust with `none` as default can be omitted,
//! like `.mel`-declared ones defaulting to `_`.

use melodium::{load_raw, LoadingConfig};
use melodium_common::descriptor::{Collection, Entry, Identifier, Parameter};
use melodium_common::executive::Value;
use std::{collections::HashMap, str::FromStr, sync::Arc};

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = option_defaults
#! version = 0.11.0
#! require = std:0.11.0 work:0.11.0 record:0.11.0

use std/engine/util::startup
use work/distant::DistantEngine
use record/audio::recordMono

treatment main()
  model distant: DistantEngine()
{
    startup()
    recordMono()

    startup.trigger -> recordMono.trigger
}
"#;

fn parameters(collection: &Collection, identifier: &str) -> HashMap<String, Parameter> {
    let identifier = Identifier::from_str(identifier).unwrap();
    match collection.get(&(&identifier).into()) {
        Some(Entry::Model(model)) => model.parameters().clone(),
        Some(Entry::Treatment(treatment)) => treatment.parameters().clone(),
        _ => panic!("{} not found", identifier),
    }
}

#[test]
fn option_parameters_default_to_none() {
    let (_, collection) = load_raw(
        Arc::new(SCRIPT.as_bytes().to_vec()),
        "main",
        LoadingConfig {
            core_packages: Vec::new(),
            search_locations: Vec::new(),
            raw_elements: Vec::new(),
        },
    )
    .into_result()
    .expect("script loads without giving options");

    // Model parameters.
    let distant = parameters(&collection, "work/distant::DistantEngine");
    assert_eq!(distant["api_url"].default(), &Some(Value::Option(None)));
    assert_eq!(distant["api_token"].default(), &Some(Value::Option(None)));
    assert_eq!(
        distant["location"].default(),
        &Some(Value::String("api".to_string()))
    );

    // Treatment parameters.
    let record = parameters(&collection, "record/audio::recordMono");
    assert_eq!(record["device"].default(), &Some(Value::Option(None)));
    assert_eq!(record["sample_rate"].default(), &Some(Value::Option(None)));
}
