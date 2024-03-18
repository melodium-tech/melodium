#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_package, mel_treatment};
use std::sync::Arc;

pub mod value;

/// JSON data.
///
/// `Json` data type contains any valid json value.
///
/// ℹ️ The traits `ToString` and `TryToString` have different behavior for conversion:
/// - `ToString`, as infaillible, will give the literal JSON object string;
/// - `TryToString`, as faillible, will give the internal string _if JSON object is only a string_, and none in the other cases.
#[mel_data(traits(ToString TryToString TryToBool TryToI64 TryToU64 TryToF64 Display))]
#[derive(Debug, Clone, Serialize)]
pub struct Json(pub serde_json::Value);

impl ToString for Json {
    fn to_string(&self) -> string {
        self.0.to_string()
    }
}

impl TryToString for Json {
    fn try_to_string(&self) -> Option<string> {
        if let Json(serde_json::Value::String(s)) = self {
            Some(s.clone())
        } else {
            None
        }
    }
}

impl TryToBool for Json {
    fn try_to_bool(&self) -> Option<bool> {
        if let Json(serde_json::Value::Bool(b)) = self {
            Some(*b)
        } else {
            None
        }
    }
}

impl TryToI64 for Json {
    fn try_to_i64(&self) -> Option<i64> {
        if let Json(serde_json::Value::Number(num)) = self {
            num.as_i64()
        } else {
            None
        }
    }
}

impl TryToU64 for Json {
    fn try_to_u64(&self) -> Option<u64> {
        if let Json(serde_json::Value::Number(num)) = self {
            num.as_u64()
        } else {
            None
        }
    }
}

impl TryToF64 for Json {
    fn try_to_f64(&self) -> Option<f64> {
        if let Json(serde_json::Value::Number(num)) = self {
            num.as_f64()
        } else {
            None
        }
    }
}

impl Display for Json {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", serde_json::ser::to_string_pretty(&self.0).unwrap())
    }
}

/// Parse string into Json data.
#[mel_function]
pub fn to_json(text: string) -> Option<Json> {
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .map(|json| Json(json))
}

/// Parse string into Json data.
///
/// `json` contains json data if input `text` contains valid json, else none.
/// `error` contains message if input `text` is not valid json, else none.
#[mel_treatment(
    input text Stream<string>
    output json Stream<Option<Json>>
    output error Stream<Option<string>>
)]
pub async fn to_json() {
    'main: while let Ok(text) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        for t in text {
            let result = serde_json::from_str::<serde_json::Value>(&t);
            match result {
                Ok(json_value) => {
                    if let (Err(_), Err(_)) = futures::join!(
                        json.send_one(Some(Arc::new(Json(json_value)) as Arc<dyn Data>).into()),
                        error.send_one(Option::<string>::None.into())
                    ) {
                        break 'main;
                    }
                }
                Err(err) => {
                    if let (Err(_), Err(_)) = futures::join!(
                        json.send_one(Option::<Arc<dyn Data>>::None.into()),
                        error.send_one(Some(err.to_string()).into())
                    ) {
                        break 'main;
                    }
                }
            }
        }
    }
}

/// Validate JSON string.
///
/// Tells wether `text` is valid JSON or not.
#[mel_treatment(
    input {content(json)} text Stream<string>
    output is_json Stream<bool>
)]
pub async fn validate() {
    while let Ok(text) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        check!(
            is_json
                .send_many(
                    text.iter()
                        .map(|t| match serde_json::from_str::<serde::de::IgnoredAny>(t) {
                            Ok(_) => true,
                            Err(_) => false,
                        })
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
        );
    }
}

mel_package!();
