use super::*;
use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_treatment};

#[mel_function]
pub fn null() -> Json {
    Json(serde_json::Value::Null)
}

/// Makes a JSON boolean value.
#[mel_function(
    generic B (ToBool)
)]
pub fn bool(value: B) -> Json {
    Json(serde_json::Value::Bool(value.to_bool()))
}

/// Makes a JSON numeric value from i64
#[mel_function(
    generic I (ToI64)
)]
pub fn number_i64(value: I) -> Json {
    Json(serde_json::Value::from(value.to_i64()))
}

/// Makes a JSON numeric value from u64
#[mel_function(
    generic U (ToU64)
)]
pub fn number_u64(value: U) -> Json {
    Json(serde_json::Value::from(value.to_u64()))
}

/// Try to make a JSON numeric value from f64
///
/// An infinite or NaN number is not a valid JSON value, and then return none value if in that case.
#[mel_function(
    generic F (ToF64)
)]
pub fn number_f64(value: F) -> Option<Json> {
    if let Some(num) = serde_json::Number::from_f64(value.to_f64()) {
        Some(Json(serde_json::Value::from(num)))
    }
    else {
        None
    }
}

/// Makes a JSON string value
#[mel_function(
    generic S (ToString)
)]
pub fn string(value: S) -> Json {
    Json(serde_json::Value::from(DataTrait::to_string(&value)))
}

