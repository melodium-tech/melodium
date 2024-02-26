use super::*;
//use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_function, mel_treatment};

/// Return json `null` value.
#[mel_function]
pub fn null() -> Json {
    Json(serde_json::Value::Null)
}

/// Makes stream of json `null` values.
#[mel_treatment(
    input ticks Stream<void>
    output nulls Stream<Json>
)]
pub async fn null() {
    while let Ok(ticks) = ticks.recv_many().await {
        check!(
            nulls
                .send_many(TransmissionValue::Other(
                    vec![Value::Data(Arc::new(Json(serde_json::Value::Null))); ticks.len()].into()
                ))
                .await
        )
    }
}

/// Makes a JSON boolean value.
#[mel_function(
    generic B (ToBool)
)]
pub fn from_bool(value: B) -> Json {
    Json(serde_json::Value::Bool(value.to_bool()))
}

/// Turns stream of boolean convertible values into json booleans.
#[mel_treatment(
    generic B (ToBool)
    input value Stream<B>
    output json Stream<Json>
)]
pub async fn from_bool() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Data(Arc::new(Json(serde_json::Value::Bool(val.to_bool())))))
                    .collect()
            ))
            .await
        )
    }
}

/// Makes a JSON numeric value from i64
#[mel_function(
    generic I (ToI64)
)]
pub fn from_number_i64(value: I) -> Json {
    Json(serde_json::Value::from(value.to_i64()))
}

/// Turns stream of i64 convertible values into json numbers.
#[mel_treatment(
    generic I (ToI64)
    input value Stream<I>
    output json Stream<Json>
)]
pub async fn from_number_i64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Data(Arc::new(Json(serde_json::Value::from(val.to_i16())))))
                    .collect()
            ))
            .await
        )
    }
}

/// Makes a JSON numeric value from u64
#[mel_function(
    generic U (ToU64)
)]
pub fn from_number_u64(value: U) -> Json {
    Json(serde_json::Value::from(value.to_u64()))
}

/// Turns stream of u64 convertible values into json numbers.
#[mel_treatment(
    generic U (ToU64)
    input value Stream<U>
    output json Stream<Json>
)]
pub async fn from_number_u64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Data(Arc::new(Json(serde_json::Value::from(val.to_u16())))))
                    .collect()
            ))
            .await
        )
    }
}

/// Try to make a JSON numeric value from f64
///
/// An infinite or NaN number is not a valid JSON value, and then return none value if in that case.
#[mel_function(
    generic F (ToF64)
)]
pub fn try_from_number_f64(value: F) -> Option<Json> {
    if let Some(num) = serde_json::Number::from_f64(value.to_f64()) {
        Some(Json(serde_json::Value::from(num)))
    } else {
        None
    }
}

/// Turns stream of f64 convertible values into json numbers.
///
/// An infinite or NaN number is not a valid JSON value, and then stream none value if in that case.
#[mel_treatment(
    generic F (ToF64)
    input value Stream<F>
    output json Stream<Option<Json>>
)]
pub async fn try_from_number_f64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(
                        |val| if let Some(num) = serde_json::Number::from_f64(val.to_f64()) {
                            Value::Option(Some(Box::new(Value::Data(Arc::new(Json(
                                serde_json::Value::from(num),
                            ))))))
                        } else {
                            Value::Option(None)
                        }
                    )
                    .collect()
            ))
            .await
        )
    }
}

/// Makes a JSON numeric value from f64
///
/// An infinite or NaN number is not a valid JSON value, and then return `replacement` value if in that case.
#[mel_function(
    generic F (ToF64)
)]
pub fn from_number_f64(value: F, replacement: Json) -> Json {
    if let Some(num) = serde_json::Number::from_f64(value.to_f64()) {
        Json(serde_json::Value::from(num))
    } else {
        replacement
    }
}

/// Turns stream of u64 convertible values into json numbers.
///
/// An infinite or NaN number is not a valid JSON value, and then stream `replacement` value if in that case.
#[mel_treatment(
    generic F (ToF64)
    input value Stream<F>
    output json Stream<Json>
)]
pub async fn from_number_f64(replacement: Json) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(
                        |val| if let Some(num) = serde_json::Number::from_f64(val.to_f64()) {
                            Value::Data(Arc::new(Json(serde_json::Value::from(num))))
                        } else {
                            Value::Data(Arc::clone(&replacement) as Arc<dyn Data>)
                        }
                    )
                    .collect()
            ))
            .await
        )
    }
}

/// Makes a JSON string value
#[mel_function(
    generic S (ToString)
)]
pub fn from_string(value: S) -> Json {
    Json(serde_json::Value::from(DataTrait::to_string(&value)))
}

/// Turns stream of string convertible values into json strings.
#[mel_treatment(
    generic S (ToString)
    input value Stream<S>
    output json Stream<Json>
)]
pub async fn from_string() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Data(Arc::new(Json(serde_json::Value::String(
                        DataTrait::to_string(&val)
                    )))))
                    .collect()
            ))
            .await
        )
    }
}

/// Makes a JSON boolean or null value.
#[mel_function(
    generic B (ToBool)
)]
pub fn from_option_bool(value: Option<B>) -> Json {
    if let Some(val) = value {
        Json(serde_json::Value::Bool(val.to_bool()))
    } else {
        Json(serde_json::Value::Null)
    }
}

/// Turns stream of boolean convertible option values into json boolean or null values.
#[mel_treatment(
    generic B (ToBool)
    input value Stream<Option<B>>
    output json Stream<Json>
)]
pub async fn from_option_bool() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Data(Arc::new(Json(match val {
                        Value::Option(Some(val)) => {
                            serde_json::Value::Bool(val.to_bool())
                        }
                        _ => serde_json::Value::Null,
                    }))))
                    .collect()
            ))
            .await
        )
    }
}

/// Makes a JSON numeric or null value from option of convertible i64 value.
#[mel_function(
    generic I (ToI64)
)]
pub fn from_option_number_i64(value: Option<I>) -> Json {
    if let Some(val) = value {
        Json(serde_json::Value::from(val.to_i64()))
    } else {
        Json(serde_json::Value::Null)
    }
}

/// Turns stream of i64 convertible option values into json numbers.
#[mel_treatment(
    generic I (ToI64)
    input value Stream<Option<I>>
    output json Stream<Json>
)]
pub async fn from_option_number_i64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Data(Arc::new(Json(match val {
                        Value::Option(Some(val)) => {
                            serde_json::Value::from(val.to_i64())
                        }
                        _ => serde_json::Value::Null,
                    }))))
                    .collect()
            ))
            .await
        )
    }
}

/// Makes a JSON numeric or null value from option of convertible u64 value.
#[mel_function(
    generic U (ToU64)
)]
pub fn from_option_number_u64(value: Option<U>) -> Json {
    if let Some(val) = value {
        Json(serde_json::Value::from(val.to_u64()))
    } else {
        Json(serde_json::Value::Null)
    }
}

/// Turns stream of u64 convertible option values into json numbers.
#[mel_treatment(
    generic U (ToU64)
    input value Stream<Option<U>>
    output json Stream<Json>
)]
pub async fn from_option_number_u64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Data(Arc::new(Json(match val {
                        Value::Option(Some(val)) => {
                            serde_json::Value::from(val.to_u64())
                        }
                        _ => serde_json::Value::Null,
                    }))))
                    .collect()
            ))
            .await
        )
    }
}

/// Try to make a JSON numeric value from option f64 convertible value
///
/// An infinite or NaN number is not a valid JSON value, and then return none value if in that case.
#[mel_function(
    generic F (ToF64)
)]
pub fn try_from_option_number_f64(value: Option<F>) -> Option<Json> {
    if let Some(val) = value {
        if let Some(num) = serde_json::Number::from_f64(val.to_f64()) {
            Some(Json(serde_json::Value::from(num)))
        } else {
            None
        }
    } else {
        Some(Json(serde_json::Value::Null))
    }
}

/// Turns stream of option f64 convertible values into json numbers.
///
/// An infinite or NaN number is not a valid JSON value, and then stream none value if in that case.
#[mel_treatment(
    generic F (ToF64)
    input value Stream<Option<F>>
    output json Stream<Option<Json>>
)]
pub async fn try_from_option_number_f64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| match val {
                        Value::Option(Some(val)) => {
                            if let Some(num) = serde_json::Number::from_f64(val.to_f64()) {
                                Value::Option(Some(Box::new(Value::Data(Arc::new(Json(
                                    serde_json::Value::from(num),
                                ))))))
                            } else {
                                Value::Option(None)
                            }
                        }
                        _ => Value::Option(Some(Box::new(Value::Data(Arc::new(Json(
                            serde_json::Value::Null
                        )))))),
                    })
                    .collect()
            ))
            .await
        )
    }
}

/// Makes a JSON numeric value from option f64 convertible value
///
/// An infinite or NaN number is not a valid JSON value, and then return `replacement` value in that case.
#[mel_function(
    generic F (ToF64)
)]
pub fn from_option_number_f64(value: Option<F>, replacement: Json) -> Json {
    if let Some(val) = value {
        if let Some(num) = serde_json::Number::from_f64(val.to_f64()) {
            Json(serde_json::Value::from(num))
        } else {
            replacement
        }
    } else {
        Json(serde_json::Value::Null)
    }
}

/// Turns stream of option u64 convertible values into json numbers.
///
/// An infinite or NaN number is not a valid JSON value, and then stream `replacement` value if in that case.
#[mel_treatment(
    generic F (ToF64)
    input value Stream<Option<F>>
    output json Stream<Json>
)]
pub async fn from_option_number_f64(replacement: Json) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            json.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| match val {
                        Value::Option(Some(val)) => {
                            if let Some(num) = serde_json::Number::from_f64(val.to_f64()) {
                                Value::Data(Arc::new(Json(serde_json::Value::from(num))))
                            } else {
                                Value::Data(Arc::clone(&replacement) as Arc<dyn Data>)
                            }
                        }
                        _ => Value::Data(Arc::new(Json(serde_json::Value::Null))),
                    })
                    .collect()
            ))
            .await
        )
    }
}
