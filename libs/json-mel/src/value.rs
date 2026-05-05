use super::*;
use melodium_macro::{check, mel_function, mel_treatment};
use std_mel::data::string_map::*;

/// Return the JSON `null` value.
#[mel_function]
pub fn null() -> Json {
    Json(serde_json::Value::Null)
}

/// Emit a JSON `null` for each tick received on `ticks`.
///
/// ```mermaid
/// graph LR
///     T("null()")
///     A["🟦 … 🟨"] -->|ticks| T
///     T -->|nulls| B["null … null"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert a bool-convertible value into a JSON boolean.
#[mel_function(
    generic B (ToBool)
)]
pub fn from_bool(value: B) -> Json {
    Json(serde_json::Value::Bool(value.to_bool()))
}

/// Convert each bool-convertible value in the stream into a JSON boolean.
///
/// ```mermaid
/// graph LR
///     T("fromBool()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|json| B["true … false"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert an i64-convertible value into a JSON number.
#[mel_function(
    generic I (ToI64)
)]
pub fn from_number_i64(value: I) -> Json {
    Json(serde_json::Value::from(value.to_i64()))
}

/// Convert each i64-convertible value in the stream into a JSON number.
///
/// ```mermaid
/// graph LR
///     T("fromNumberI64()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|json| B["🟦 … 🟨"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert a u64-convertible value into a JSON number.
#[mel_function(
    generic U (ToU64)
)]
pub fn from_number_u64(value: U) -> Json {
    Json(serde_json::Value::from(value.to_u64()))
}

/// Convert each u64-convertible value in the stream into a JSON number.
///
/// ```mermaid
/// graph LR
///     T("fromNumberU64()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|json| B["🟦 … 🟨"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert an f64-convertible value into a JSON number, returning `none` if the value is infinite or NaN.
///
/// ⚠️ Infinite and NaN values are not valid JSON; use `from_number_f64` if you need a fallback instead of `none`.
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

/// Convert each f64-convertible value in the stream into a JSON number, emitting `none` for infinite or NaN values.
///
/// ⚠️ Infinite and NaN values are not valid JSON; use `fromNumberF64` if you need a fallback instead of `none`.
///
/// ```mermaid
/// graph LR
///     T("tryFromNumberF64()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|json| B["〈🟦〉 … 〈none〉"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert an f64-convertible value into a JSON number, using `replacement` if the value is infinite or NaN.
///
/// ⚠️ Infinite and NaN values are not valid JSON; `replacement` must itself be a valid JSON value.
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

/// Convert each f64-convertible value in the stream into a JSON number, emitting `replacement` for infinite or NaN values.
///
/// ⚠️ Infinite and NaN values are not valid JSON; `replacement` must itself be a valid JSON value.
///
/// ```mermaid
/// graph LR
///     T("fromNumberF64(replacement=🟥)")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|json| B["🟦 … 🟥"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert a string-convertible value into a JSON string.
#[mel_function(
    generic S (ToString)
)]
pub fn from_string(value: S) -> Json {
    Json(serde_json::Value::from(DataTrait::to_string(&value)))
}

/// Convert each string-convertible value in the stream into a JSON string.
///
/// ```mermaid
/// graph LR
///     T("fromString()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|json| B["🟦 … 🟨"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert a `StringMap` into a JSON object where every value is a JSON string.
#[mel_function]
pub fn from_string_map(map: StringMap) -> Json {
    Json(serde_json::Value::Object(
        map.map
            .iter()
            .map(|(key, val)| (key.into(), serde_json::Value::String(val.into())))
            .collect(),
    ))
}

/// Convert each `StringMap` in the stream into a JSON object where every value is a JSON string.
///
/// ```mermaid
/// graph LR
///     T("fromStringMap()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|json| B["🟦 … 🟨"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<StringMap>
    output json Stream<Json>
)]
pub async fn from_string_map() {
    while let Ok(value) = value.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<StringMap>()
            .unwrap()
    }) {
        let object = Json(serde_json::Value::Object(
            value
                .map
                .iter()
                .map(|(key, val)| (key.into(), serde_json::Value::String(val.into())))
                .collect(),
        ));
        check!(json.send_one(Value::Data(Arc::new(object))).await)
    }
}

/// Convert an optional bool-convertible value into a JSON boolean, or `null` if `value` is `none`.
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

/// Convert each optional bool-convertible value in the stream into a JSON boolean, or `null` for `none` elements.
///
/// ```mermaid
/// graph LR
///     T("fromOptionBool()")
///     A["〈🟦〉 … 〈none〉"] -->|value| T
///     T -->|json| B["true … null"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert an optional i64-convertible value into a JSON number, or `null` if `value` is `none`.
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

/// Convert each optional i64-convertible value in the stream into a JSON number, or `null` for `none` elements.
///
/// ```mermaid
/// graph LR
///     T("fromOptionNumberI64()")
///     A["〈🟦〉 … 〈none〉"] -->|value| T
///     T -->|json| B["🟦 … null"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert an optional u64-convertible value into a JSON number, or `null` if `value` is `none`.
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

/// Convert each optional u64-convertible value in the stream into a JSON number, or `null` for `none` elements.
///
/// ```mermaid
/// graph LR
///     T("fromOptionNumberU64()")
///     A["〈🟦〉 … 〈none〉"] -->|value| T
///     T -->|json| B["🟦 … null"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert an optional f64-convertible value into a JSON number, `null` if `value` is `none`, or `none` if the number is infinite or NaN.
///
/// ⚠️ Infinite and NaN values are not valid JSON and result in `none` being returned, not a JSON value.
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

/// Convert each optional f64-convertible value in the stream into a JSON number, `null` for `none` elements, or `none` for infinite/NaN values.
///
/// ⚠️ Infinite and NaN values are not valid JSON and produce `none` on `json`, not a JSON `null`.
///
/// ```mermaid
/// graph LR
///     T("tryFromOptionNumberF64()")
///     A["〈🟦〉 … 〈none〉"] -->|value| T
///     T -->|json| B["〈🟦〉 … 〈null〉 … 〈none〉"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert an optional f64-convertible value into a JSON number, `null` if `value` is `none`, or `replacement` if the number is infinite or NaN.
///
/// ⚠️ Infinite and NaN values are not valid JSON; `replacement` must itself be a valid JSON value.
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

/// Convert each optional f64-convertible value in the stream into a JSON number, `null` for `none` elements, or `replacement` for infinite/NaN values.
///
/// ⚠️ Infinite and NaN values are not valid JSON; `replacement` must itself be a valid JSON value.
///
/// ```mermaid
/// graph LR
///     T("fromOptionNumberF64(replacement=🟥)")
///     A["〈🟦〉 … 〈none〉"] -->|value| T
///     T -->|json| B["🟦 … null … 🟥"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
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

/// Convert an optional string-convertible value into a JSON string, or `null` if `value` is `none`.
#[mel_function(
    generic S (ToString)
)]
pub fn from_option_string(value: Option<S>) -> Json {
    if let Some(val) = value {
        Json(serde_json::Value::String(DataTrait::to_string(&val)))
    } else {
        Json(serde_json::Value::Null)
    }
}

/// Convert each optional string-convertible value in the stream into a JSON string, or `null` for `none` elements.
///
/// ```mermaid
/// graph LR
///     T("fromOptionString()")
///     A["〈🟦〉 … 〈none〉"] -->|value| T
///     T -->|json| B["🟦 … null"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic S (ToString)
    input value Stream<Option<S>>
    output json Stream<Json>
)]
pub async fn from_option_string() {
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
                            serde_json::Value::String(DataTrait::to_string(&*val))
                        }
                        _ => serde_json::Value::Null,
                    }))))
                    .collect()
            ))
            .await
        )
    }
}

/// Return `true` if `value` is JSON `null`.
#[mel_function]
pub fn is_null(value: Json) -> bool {
    value.0.is_null()
}

/// Emit `true` for each JSON value in the stream that is `null`, `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isNull()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_null| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_null Stream<bool>
)]
pub async fn is_null() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_null
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_null())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Return `true` if `value` is a JSON boolean.
#[mel_function]
pub fn is_bool(value: Json) -> bool {
    value.0.is_boolean()
}

/// Emit `true` for each JSON value in the stream that is a boolean, `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isBool()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_bool| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_bool Stream<bool>
)]
pub async fn is_bool() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_bool
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_boolean())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Return `true` if `value` is a JSON string.
#[mel_function]
pub fn is_string(value: Json) -> bool {
    value.0.is_string()
}

/// Emit `true` for each JSON value in the stream that is a string, `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isString()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_string| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_string Stream<bool>
)]
pub async fn is_string() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_string
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_string())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Return `true` if `value` is any JSON number (integer or float).
#[mel_function]
pub fn is_number(value: Json) -> bool {
    value.0.is_number()
}

/// Emit `true` for each JSON value in the stream that is any number (integer or float), `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isNumber()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_number| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_number Stream<bool>
)]
pub async fn is_number() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_number
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_number())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Return `true` if `value` is a JSON number representable as i64.
#[mel_function]
pub fn is_i64(value: Json) -> bool {
    value.0.is_i64()
}

/// Emit `true` for each JSON value in the stream that is representable as i64, `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isI64()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_i64| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_i64 Stream<bool>
)]
pub async fn is_i64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_i64
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_i64())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Return `true` if `value` is a JSON number representable as u64.
#[mel_function]
pub fn is_u64(value: Json) -> bool {
    value.0.is_u64()
}

/// Emit `true` for each JSON value in the stream that is representable as u64, `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isU64()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_u64| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_u64 Stream<bool>
)]
pub async fn is_u64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_u64
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_u64())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Return `true` if `value` is a JSON number representable as f64.
#[mel_function]
pub fn is_f64(value: Json) -> bool {
    value.0.is_f64()
}

/// Emit `true` for each JSON value in the stream that is representable as f64, `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isF64()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_f64| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_f64 Stream<bool>
)]
pub async fn is_f64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_f64
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_f64())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Return `true` if `value` is a JSON array.
#[mel_function]
pub fn is_vec(value: Json) -> bool {
    value.0.is_array()
}

/// Emit `true` for each JSON value in the stream that is an array, `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isVector()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_vector| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_vector Stream<bool>
)]
pub async fn is_vector() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_vector
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_array())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Return `true` if `value` is a JSON object.
#[mel_function]
pub fn is_object(value: Json) -> bool {
    value.0.is_object()
}

/// Emit `true` for each JSON value in the stream that is an object, `false` otherwise.
///
/// ```mermaid
/// graph LR
///     T("isObject()")
///     A["🟦 … 🟨"] -->|value| T
///     T -->|is_object| B["false … true"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Json>
    output is_object Stream<bool>
)]
pub async fn is_object() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            is_object
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_object())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect()
                ))
                .await
        )
    }
}
