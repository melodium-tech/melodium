#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};
use melodium_core::{*, executive::*};
use melodium_macro::{check, mel_package, mel_treatment, mel_data};

#[mel_data(
    traits (ToString Hash Display)
)]
#[derive(Debug, Clone)]
pub struct Json {
    json: String,
}

impl ToString for Json {
    fn to_string(&self) -> string {
        self.json.clone()
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

/// Execute query on JSON string.
///
/// For each string coming through `json`, `query` is applied.
/// `parsed` and `error` contains the results and errors of the parsing.
/// An input string not being valid JSON is considered as an error.
///
/// `failures` is emitted only if the query provided is not valid [jq/jaq syntax](https://jqlang.github.io/jq/manual/v1.6/).
#[mel_treatment(
    input {content(json)} json Stream<string>
    output {content(json)} parsed Stream<Vec<string>>
    output error Stream<Vec<string>>
    output failures Block<Vec<string>>
)]
pub async fn query(#[mel(content(jq))] query: string) {
    let mut defs = ParseCtx::new(Vec::new());
    defs.insert_natives(jaq_core::core());
    defs.insert_defs(jaq_std::std());

    let (filter, errs) = jaq_parse::parse(&query, jaq_parse::main());
    if !errs.is_empty() {
        let _ = failures
            .send_one(Value::Vec(
                errs.into_iter().map(|e| e.to_string().into()).collect(),
            ))
            .await;
    } else {
        let filter = defs.compile(filter.unwrap());
        if !defs.errs.is_empty() {
            let _ = failures
                .send_one(Value::Vec(
                    defs.errs
                        .into_iter()
                        .map(|e| e.0.to_string().into())
                        .collect(),
                ))
                .await;
        } else {
            while let Ok(json) = json
                .recv_one()
                .await
                .map(|val| GetData::<string>::try_data(val).unwrap())
            {
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(value) => {
                        let inputs = RcIter::new(core::iter::empty());
                        let mut outputs = Vec::new();
                        let mut errors = Vec::new();
                        for output in filter.run((Ctx::new([], &inputs), Val::from(value))) {
                            match output {
                                Ok(output) => {
                                    outputs.push(output.to_string());
                                }
                                Err(err) => {
                                    errors.push(err.to_string());
                                }
                            }
                        }
                        if let (Err(_), Err(_)) = (
                            parsed.send_one(outputs.into()).await,
                            error.send_one(errors.into()).await,
                        ) {
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = error.send_one(vec![err.to_string()].into()).await;
                    }
                }
            }
        }
    }
}

mel_package!();
