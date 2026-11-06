use super::*;
use melodium_core::common::descriptor::DataType;
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input ticks Stream<void>
    output nulls Stream<Json>
)]
pub async fn null() {
    while let Ok(ticks) = ticks.recv_many().await {
        check!(
            nulls
                .send_many_as(vec![Arc::new(Json(serde_json::Value::Null)); ticks.len()])
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(|val| Arc::new(Json(serde_json::Value::Bool(val.to_bool()))))
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(|val| Arc::new(Json(serde_json::Value::from(val.to_i64()))))
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(|val| Arc::new(Json(serde_json::Value::from(val.to_u64()))))
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(
                        |val| if let Some(num) = serde_json::Number::from_f64(val.to_f64()) {
                            Arc::new(Json(serde_json::Value::from(num)))
                        } else {
                            Arc::clone(&replacement)
                        }
                    )
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(
                        |val| Arc::new(Json(serde_json::Value::String(DataTrait::to_string(&val))))
                    )
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input value Stream<StringMap>
    output json Stream<Json>
)]
pub async fn from_string_map() {
    while let Ok(value) = value.recv_one_as::<Arc<StringMap>>().await {
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(|val| Arc::new(Json(match val {
                        Value::Option(Some(val)) => {
                            serde_json::Value::Bool(val.to_bool())
                        }
                        _ => serde_json::Value::Null,
                    })))
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(|val| Arc::new(Json(match val {
                        Value::Option(Some(val)) => {
                            serde_json::Value::from(val.to_i64())
                        }
                        _ => serde_json::Value::Null,
                    })))
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(|val| Arc::new(Json(match val {
                        Value::Option(Some(val)) => {
                            serde_json::Value::from(val.to_u64())
                        }
                        _ => serde_json::Value::Null,
                    })))
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(|val| match val {
                        Value::Option(Some(val)) => {
                            if let Some(num) = serde_json::Number::from_f64(val.to_f64()) {
                                Arc::new(Json(serde_json::Value::from(num)))
                            } else {
                                Arc::clone(&replacement)
                            }
                        }
                        _ => Arc::new(Json(serde_json::Value::Null)),
                    })
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
            json.send_many_as(
                values
                    .into_iter()
                    .map(|val| Arc::new(Json(match val {
                        Value::Option(Some(val)) => {
                            serde_json::Value::String(DataTrait::to_string(&*val))
                        }
                        _ => serde_json::Value::Null,
                    })))
                    .collect::<Vec<_>>()
            )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_null())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_boolean())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_string())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_number())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_i64())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_u64())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_f64())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_array())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
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
                .send_many_as(
                    values
                        .into_iter()
                        .map(|val| match val {
                            Value::Data(val) => val
                                .downcast_arc::<Json>()
                                .map(|json| json.0.is_object())
                                .unwrap_or(false),
                            _ => false,
                        })
                        .collect::<Vec<_>>()
                )
                .await
        )
    }
}

/// Return the value of `key` in the JSON object `json`, or none if `json` is not an object or
/// has no `key`.
#[mel_function]
pub fn get(json: Json, key: string) -> Option<Json> {
    json.0.get(&key).cloned().map(Json)
}

/// Emit the value of `key` for each JSON value in the stream, or none if it is not an object
/// or has no `key`.
///
/// ```mermaid
/// graph LR
///     T("get(key=\"name\")")
///     A["{name: 🟦} … 🟨"] -->|json| T
///     T -->|value| B["〈🟦〉 … 〈none〉"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input json Stream<Json>
    output value Stream<Option<Json>>
)]
pub async fn get(key: string) {
    while let Ok(json) = json.recv_one_as::<Arc<Json>>().await {
        check!(value.send_one(found(json.0.get(&key))).await)
    }
}

/// Return the element at `index` of the JSON array `json`, from 0, or none if `json` is not an
/// array or is shorter.
#[mel_function]
pub fn index(json: Json, index: u64) -> Option<Json> {
    usize::try_from(index)
        .ok()
        .and_then(|index| json.0.get(index))
        .cloned()
        .map(Json)
}

/// Emit the element at `index` for each JSON value in the stream, from 0, or none if it is not
/// an array or is shorter.
///
/// ```mermaid
/// graph LR
///     T("index(index=1)")
///     A["[🟨, 🟦] … 🟨"] -->|json| T
///     T -->|value| B["〈🟦〉 … 〈none〉"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input json Stream<Json>
    output value Stream<Option<Json>>
)]
pub async fn index(index: u64) {
    let index = usize::try_from(index).ok();
    while let Ok(json) = json.recv_one_as::<Arc<Json>>().await {
        check!(
            value
                .send_one(found(index.and_then(|index| json.0.get(index))))
                .await
        )
    }
}

/// Return the value at `pointer` within `json`, or none if there is none.
///
/// `pointer` is a JSON Pointer (RFC 6901), such as `"/hit/download/url"`, `"/parts/0"`, or
/// `""` for `json` itself. In names, `~1` stands for `/` and `~0` for `~`.
#[mel_function]
pub fn pointer(json: Json, pointer: string) -> Option<Json> {
    json.0.pointer(&pointer).cloned().map(Json)
}

/// Emit the value at `pointer` for each JSON value in the stream, or none if there is none.
///
/// `pointer` is a JSON Pointer (RFC 6901), such as `"/hit/download/url"`, `"/parts/0"`, or
/// `""` for the value itself. In names, `~1` stands for `/` and `~0` for `~`.
///
/// ```mermaid
/// graph LR
///     T("pointer(pointer=\"/a/0\")")
///     A["{a: [🟦]} … 🟨"] -->|json| T
///     T -->|value| B["〈🟦〉 … 〈none〉"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input json Stream<Json>
    output value Stream<Option<Json>>
)]
pub async fn pointer(pointer: string) {
    while let Ok(json) = json.recv_one_as::<Arc<Json>>().await {
        check!(value.send_one(found(json.0.pointer(&pointer))).await)
    }
}

/// Return an empty JSON object.
#[mel_function]
pub fn object() -> Json {
    Json(serde_json::Value::Object(serde_json::Map::new()))
}

/// Return `object` with `key` set to `value`, replacing the value it had.
///
/// If `object` is not a JSON object, it is returned unchanged.
#[mel_function]
pub fn insert(object: Json, key: string, value: Json) -> Json {
    inserted(object.0, key, value.0)
}

/// Emit each JSON object of `object` with `key` set to the value of `value` coming with it,
/// replacing the value it had.
///
/// Values that are not JSON objects are emitted unchanged.
///
/// ```mermaid
/// graph LR
///     T("insert(key=\"name\")")
///     A["{} … {name: 🟨}"] -->|object| T
///     V["🟦 … 🟦"] -->|value| T
///     T -->|json| B["{name: 🟦} … {name: 🟦}"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style V fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input object Stream<Json>
    input value Stream<Json>
    output json Stream<Json>
)]
pub async fn insert(key: string) {
    while let (Ok(object), Ok(value)) = (
        object.recv_one_as::<Arc<Json>>().await,
        value.recv_one_as::<Arc<Json>>().await,
    ) {
        check!(
            json.send_one(Value::Data(Arc::new(inserted(
                object.0.clone(),
                key.clone(),
                value.0.clone()
            ))))
            .await
        )
    }
}

/// Return a JSON array of `values`.
#[mel_function]
pub fn array(values: Vec<Json>) -> Json {
    Json(serde_json::Value::Array(
        values.into_iter().map(|json| json.0).collect(),
    ))
}

/// Emit a JSON array of all the values of the stream, once it ends.
///
/// ```mermaid
/// graph LR
///     T("array()")
///     A["🟦 … 🟨"] -->|values| T
///     T -->|array| B["〈[🟦, …, 🟨]〉"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input values Stream<Json>
    output array Block<Json>
)]
pub async fn array() {
    let mut elements = Vec::new();
    while let Ok(json) = values.recv_one_as::<Arc<Json>>().await {
        elements.push(json.0.clone());
    }
    let _ = array
        .send_one(Value::Data(Arc::new(Json(serde_json::Value::Array(
            elements,
        )))))
        .await;
}

/// Return `array` with `value` added at its end.
///
/// If `array` is not a JSON array, it is returned unchanged.
#[mel_function]
pub fn push(array: Json, value: Json) -> Json {
    pushed(array.0, value.0)
}

/// Emit each JSON array of `array` with the value of `value` coming with it added at its end.
///
/// Values that are not JSON arrays are emitted unchanged.
///
/// ```mermaid
/// graph LR
///     T("push()")
///     A["[🟨] … []"] -->|array| T
///     V["🟦 … 🟦"] -->|value| T
///     T -->|json| B["[🟨, 🟦] … [🟦]"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style V fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input array Stream<Json>
    input value Stream<Json>
    output json Stream<Json>
)]
pub async fn push() {
    while let (Ok(array), Ok(value)) = (
        array.recv_one_as::<Arc<Json>>().await,
        value.recv_one_as::<Arc<Json>>().await,
    ) {
        check!(
            json.send_one(Value::Data(Arc::new(pushed(
                array.0.clone(),
                value.0.clone()
            ))))
            .await
        )
    }
}

/// Return `json` as a value of type `T`, or none if it cannot be one.
///
/// Numbers, booleans, strings and `null` (for `void`) become the core types they fit in,
/// arrays become vectors, `null` becomes `none` for options, and data types are read as
/// they define, such as `StringMap` from an object of strings.
/// Secrets are never read from JSON.
#[mel_function(
    generic T (Deserialize)
)]
pub fn from_json(json: Json) -> Option<T> {
    generics
        .get("T")
        .and_then(|datatype| value_from_json(datatype, &json.0))
}

/// Emit each JSON value of the stream as a value of type `T`, or none if it cannot be one.
///
/// Numbers, booleans, strings and `null` (for `void`) become the core types they fit in,
/// arrays become vectors, `null` becomes `none` for options, and data types are read as
/// they define, such as `StringMap` from an object of strings.
/// Secrets are never read from JSON.
///
/// ```mermaid
/// graph LR
///     T("fromJson<u64>()")
///     A["42 … \"text\""] -->|json| T
///     T -->|value| B["〈42〉 … 〈none〉"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T (Deserialize)
    input json Stream<Json>
    output value Stream<Option<T>>
)]
pub async fn from_json() {
    let datatype = T;
    while let Ok(json) = json.recv_one_as::<Arc<Json>>().await {
        check!(
            value
                .send_one(Value::Option(
                    value_from_json(&datatype, &json.0).map(Box::new)
                ))
                .await
        )
    }
}

/// Return `value` as JSON, or none if it cannot be written as JSON, such as secrets.
///
/// Numbers, booleans, strings, bytes and characters become JSON scalars, `void` and `none`
/// become `null`, vectors become arrays, and data types are written as they define, such as
/// `StringMap` as an object of strings.
#[mel_function(
    generic T (Serialize)
)]
pub fn into_json(value: T) -> Option<Json> {
    json_from_value(&value).map(Json)
}

/// Emit each value of the stream as JSON, or none if it cannot be written as JSON, such as
/// secrets.
///
/// Numbers, booleans, strings, bytes and characters become JSON scalars, `void` and `none`
/// become `null`, vectors become arrays, and data types are written as they define, such as
/// `StringMap` as an object of strings.
///
/// ```mermaid
/// graph LR
///     T("intoJson<u64>()")
///     A["42 … 7"] -->|value| T
///     T -->|json| B["〈42〉 … 〈7〉"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T (Serialize)
    input value Stream<T>
    output json Stream<Option<Json>>
)]
pub async fn into_json() {
    while let Ok(value) = value.recv_one().await {
        check!(
            json.send_one(Value::Option(
                json_from_value(&value).map(|json| Box::new(Value::Data(Arc::new(Json(json)))))
            ))
            .await
        )
    }
}

/// Gives `json`, when found, as an optional `Json` value.
fn found(json: Option<&serde_json::Value>) -> Value {
    Value::Option(
        json.map(|json| Box::new(Value::Data(Arc::new(Json(json.clone())) as Arc<dyn Data>))),
    )
}

/// Gives `object` with `key` set to `value`, or unchanged when it is not an object.
fn inserted(object: serde_json::Value, key: String, value: serde_json::Value) -> Json {
    match object {
        serde_json::Value::Object(mut map) => {
            map.insert(key, value);
            Json(serde_json::Value::Object(map))
        }
        other => Json(other),
    }
}

/// Gives `array` with `value` at its end, or unchanged when it is not an array.
fn pushed(array: serde_json::Value, value: serde_json::Value) -> Json {
    match array {
        serde_json::Value::Array(mut elements) => {
            elements.push(value);
            Json(serde_json::Value::Array(elements))
        }
        other => Json(other),
    }
}

/// Reads `json` as a value of `datatype`.
pub(crate) fn value_from_json(datatype: &DataType, json: &serde_json::Value) -> Option<Value> {
    fn int<T: TryFrom<i64>>(json: &serde_json::Value) -> Option<T> {
        json.as_i64().and_then(|n| T::try_from(n).ok())
    }
    fn uint<T: TryFrom<u64>>(json: &serde_json::Value) -> Option<T> {
        json.as_u64().and_then(|n| T::try_from(n).ok())
    }

    if datatype.contains_secret() {
        return None;
    }
    Some(match datatype {
        DataType::Void => json.is_null().then_some(Value::Void(()))?,
        DataType::I8 => Value::I8(int(json)?),
        DataType::I16 => Value::I16(int(json)?),
        DataType::I32 => Value::I32(int(json)?),
        DataType::I64 => Value::I64(json.as_i64()?),
        DataType::I128 => Value::I128(json.as_i64()?.into()),
        DataType::U8 => Value::U8(uint(json)?),
        DataType::U16 => Value::U16(uint(json)?),
        DataType::U32 => Value::U32(uint(json)?),
        DataType::U64 => Value::U64(json.as_u64()?),
        DataType::U128 => Value::U128(json.as_u64()?.into()),
        DataType::F32 => Value::F32(json.as_f64()? as f32),
        DataType::F64 => Value::F64(json.as_f64()?),
        DataType::Bool => Value::Bool(json.as_bool()?),
        DataType::Byte => Value::Byte(uint(json)?),
        DataType::Char => {
            let mut chars = json.as_str()?.chars();
            match (chars.next(), chars.next()) {
                (Some(char), None) => Value::Char(char),
                _ => return None,
            }
        }
        DataType::String => Value::String(json.as_str()?.to_string()),
        DataType::Vec(inner) => Value::Vec(
            json.as_array()?
                .iter()
                .map(|element| value_from_json(inner, element))
                .collect::<Option<Vec<_>>>()?,
        ),
        DataType::Option(inner) => Value::Option(if json.is_null() {
            None
        } else {
            Some(Box::new(value_from_json(inner, json)?))
        }),
        // String maps are read from objects of strings, as headers usually are.
        DataType::Data(data) if *data.identifier() == __mel_data_StringMap::identifier() => {
            Value::Data(Arc::new(StringMap::new_with(
                json.as_object()?
                    .iter()
                    .map(|(key, value)| Some((key.clone(), value.as_str()?.to_string())))
                    .collect::<Option<_>>()?,
            )))
        }
        DataType::Data(data) => data
            .deserialize(&mut <dyn erased_serde::Deserializer>::erase(json.clone()))
            .ok()?,
        _ => return None,
    })
}

/// Writes `value` as JSON.
pub(crate) fn json_from_value(value: &Value) -> Option<serde_json::Value> {
    use serde_json::Value as JsonValue;
    Some(match value {
        Value::Void(_) => JsonValue::Null,
        Value::I8(n) => (*n).into(),
        Value::I16(n) => (*n).into(),
        Value::I32(n) => (*n).into(),
        Value::I64(n) => (*n).into(),
        Value::I128(n) => i64::try_from(*n).ok()?.into(),
        Value::U8(n) => (*n).into(),
        Value::U16(n) => (*n).into(),
        Value::U32(n) => (*n).into(),
        Value::U64(n) => (*n).into(),
        Value::U128(n) => u64::try_from(*n).ok()?.into(),
        Value::F32(n) => serde_json::Number::from_f64((*n).into())?.into(),
        Value::F64(n) => serde_json::Number::from_f64(*n)?.into(),
        Value::Bool(b) => (*b).into(),
        Value::Byte(b) => (*b).into(),
        Value::Char(c) => c.to_string().into(),
        Value::String(s) => s.clone().into(),
        Value::Vec(values) => JsonValue::Array(
            values
                .iter()
                .map(json_from_value)
                .collect::<Option<Vec<_>>>()?,
        ),
        Value::Packed(packed) => JsonValue::Array(
            packed
                .clone()
                .into_values()
                .iter()
                .map(json_from_value)
                .collect::<Option<Vec<_>>>()?,
        ),
        Value::Option(None) => JsonValue::Null,
        Value::Option(Some(value)) => json_from_value(value)?,
        Value::Secret(_) => return None,
        Value::Data(data) => match Arc::clone(data).downcast_arc::<StringMap>() {
            Ok(map) => JsonValue::Object(
                map.map
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone().into()))
                    .collect(),
            ),
            Err(_) => serde_json::to_value(value).ok()?,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn values_are_read_from_json() {
        let answer =
            json!({"hit": {"size": 42, "sha256": "abc", "parts": [1, 2, 3], "none": null}});
        assert_eq!(
            answer
                .pointer("/hit/size")
                .and_then(|json| value_from_json(&DataType::U64, json)),
            Some(Value::U64(42))
        );
        assert_eq!(
            value_from_json(&DataType::String, &answer["hit"]["sha256"]),
            Some(Value::String("abc".to_string()))
        );
        assert_eq!(
            value_from_json(
                &DataType::Vec(Box::new(DataType::U8)),
                &answer["hit"]["parts"]
            ),
            Some(Value::Vec(vec![Value::U8(1), Value::U8(2), Value::U8(3)]))
        );
        assert_eq!(
            value_from_json(
                &DataType::Option(Box::new(DataType::U8)),
                &answer["hit"]["none"]
            ),
            Some(Value::Option(None))
        );
        let headers = value_from_json(
            &DataType::Data(__mel_data_StringMap::descriptor()),
            &json!({"Range": "bytes=0-"}),
        );
        assert!(
            matches!(&headers, Some(Value::Data(data)) if data.clone().downcast_arc::<StringMap>().unwrap().map["Range"] == "bytes=0-")
        );
        assert_eq!(
            value_from_json(
                &DataType::Data(__mel_data_StringMap::descriptor()),
                &json!({"Range": 0})
            ),
            None
        );
        // Too large, wrong type, or not one character.
        assert_eq!(value_from_json(&DataType::I8, &json!(300)), None);
        assert_eq!(value_from_json(&DataType::U64, &json!("42")), None);
        assert_eq!(value_from_json(&DataType::Char, &json!("ab")), None);
        assert_eq!(
            value_from_json(&DataType::Vec(Box::new(DataType::U8)), &json!([1, "two"])),
            None
        );
    }

    #[test]
    fn values_are_written_as_json() {
        assert_eq!(
            json_from_value(&Value::Vec(vec![
                Value::Byte(7),
                Value::String("text".to_string())
            ])),
            Some(json!([7, "text"]))
        );
        assert_eq!(
            json_from_value(&Value::Packed(PackedArray::Byte(Arc::new(vec![1, 2])))),
            Some(json!([1, 2]))
        );
        assert_eq!(json_from_value(&Value::Option(None)), Some(json!(null)));
        assert_eq!(json_from_value(&Value::F64(f64::NAN)), None);
        assert_eq!(
            json_from_value(&Value::Data(Arc::new(Json(json!({"a": 1}))))),
            Some(json!({"a": 1}))
        );
        assert_eq!(
            json_from_value(&Value::Data(Arc::new(StringMap::new_with(
                [("Range".to_string(), "bytes=0-".to_string())].into()
            )))),
            Some(json!({"Range": "bytes=0-"}))
        );
    }

    #[test]
    fn objects_and_arrays_are_built() {
        let object = inserted(json!({"a": 1}), "b".to_string(), json!([2]));
        assert_eq!(object.0, json!({"a": 1, "b": [2]}));
        assert_eq!(
            inserted(json!([1]), "b".to_string(), json!(2)).0,
            json!([1])
        );
        assert_eq!(pushed(json!([1]), json!({"c": 3})).0, json!([1, {"c": 3}]));
        assert_eq!(pushed(json!("text"), json!(2)).0, json!("text"));
    }
}
