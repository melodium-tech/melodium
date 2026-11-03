//! The `json` package reads fields of JSON values, builds objects and arrays, and converts
//! JSON values to and from typed values.

use super::common;
use std::collections::HashMap;

const VALUES: &str = include_str!("scripts/json_values.mel");

#[test]
fn json_values_are_read_and_built() {
    let (logs, _) = common::run(VALUES, HashMap::new());
    let message = |label: &str| common::message(&logs, label).to_string();

    assert_eq!(message("size"), "42");
    assert_eq!(message("range"), "bytes=0-");
    assert_eq!(message("second"), r#"{"part_number":2}"#);
    assert_eq!(message("request"), r#"{"name":"cargo"}"#);
    assert_eq!(message("built"), r#"[{"part_number":"one"}]"#);
}
