use super::*;
use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_treatment};

#[mel_function]
pub fn null() -> Json {
    Json(serde_json::Value::Null)
}
