use super::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Return JSON `null` value.
#[mel_function]
pub fn null() -> Json {
    Json(serde_json::Value::Null)
}

/// Makes stream of JSON `null` values.
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

/// Turns stream of boolean convertible values into JSON booleans.
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

/// Turns stream of i64 convertible values into JSON numbers.
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

/// Turns stream of u64 convertible values into JSON numbers.
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

/// Turns stream of f64 convertible values into JSON numbers.
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

/// Turns stream of u64 convertible values into JSON numbers.
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

/// Turns stream of string convertible values into JSON strings.
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
///
/// If `value` is some boolean, it is turned into JSON boolean, else if `value` is `none`, `null` JSON value is returned.
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

/// Turns stream of boolean convertible option values into JSON boolean or null values.
///
/// When `value` is some boolean, it is turned into JSON boolean, else if `value` is `none`, `null` JSON value is streamed.
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
///
/// If `value` is some number, it is turned into JSON, else if `value` is `none`, `null` JSON value is returned.
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

/// Turns stream of i64 convertible option values into JSON numbers.
///
/// When `value` is some number, it is turned into JSON, else if `value` is `none`, `null` JSON value is streamed.
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
///
/// If `value` is some number, it is turned into JSON, else if `value` is `none`, `null` JSON value is returned.
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

/// Turns stream of u64 convertible option values into JSON numbers.
///
/// When `value` is some number, it is turned into JSON, else if `value` is `none`, `null` JSON value is streamed.
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
/// If `value` is some number, it is turned into JSON, else if `value` is `none`, `null` JSON value is returned.
/// An infinite or NaN number is not a valid JSON value, and then return none value in that case.
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

/// Turns stream of option f64 convertible values into JSON numbers.
///
/// If `value` is some number, it is turned into JSON, else if `value` is `none`, `null` JSON value is streamed.
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
/// If `value` is some number, it is turned into JSON, else if `value` is `none`, `null` JSON value is returned.
/// An infinite or NaN number is not a valid JSON value, then `replacement` value is used in that case.
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

/// Turns stream of option u64 convertible values into JSON numbers.
///
/// If `value` is some number, it is turned into JSON, else if `value` is `none`, `null` JSON value is streamed.
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

/// Makes a JSON string or null value.
///
/// If `value` is some string, it is turned into JSON string, else if `value` is `none`, `null` JSON value is returned.
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

/// Turns stream of string convertible option values into JSON string or null values.
///
/// When `value` is some string, it is turned into JSON string, else if `value` is `none`, `null` JSON value is streamed.
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

/// Tells if JSON value is null.
#[mel_function]
pub fn is_null(value: Json) -> bool {
    value.0.is_null()
}

/// Determine if streamed JSON values are null.
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

/// Tells if JSON value is boolean.
#[mel_function]
pub fn is_bool(value: Json) -> bool {
    value.0.is_boolean()
}

/// Determine if streamed JSON values are booleans.
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

/// Tells if JSON value is a string.
#[mel_function]
pub fn is_string(value: Json) -> bool {
    value.0.is_string()
}

/// Determine if streamed JSON values are strings.
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

/// Tells if JSON value is a number.
#[mel_function]
pub fn is_number(value: Json) -> bool {
    value.0.is_number()
}

/// Determine if streamed JSON values are numbers.
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

/// Tells if JSON value is a i64 number.
#[mel_function]
pub fn is_i64(value: Json) -> bool {
    value.0.is_i64()
}

/// Determine if streamed JSON values are i64 numbers.
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

/// Tells if JSON value is a u64 number.
#[mel_function]
pub fn is_u64(value: Json) -> bool {
    value.0.is_u64()
}

/// Determine if streamed JSON values are u64 numbers.
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

/// Tells if JSON value is a f64 number.
#[mel_function]
pub fn is_f64(value: Json) -> bool {
    value.0.is_f64()
}

/// Determine if streamed JSON values are f64 numbers.
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

/// Tells if JSON value is a vector.
#[mel_function]
pub fn is_vec(value: Json) -> bool {
    value.0.is_array()
}

/// Determine if streamed JSON values are vectors.
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

/// Tells if JSON value is an object.
#[mel_function]
pub fn is_object(value: Json) -> bool {
    value.0.is_object()
}

/// Determine if streamed JSON values are objects.
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
