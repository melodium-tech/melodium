#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::*;
use melodium_macro::{mel_package, mel_treatment};
use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};

/// Execute query on JSON string.
/// 
/// For each string coming through `json`, `query` is applied.
/// `parsed` and `error` contains the results and errors of the parsing.
/// An input string not being valid JSON is considered as an error.
/// 
/// `failures` is emitted only if the query provided is not valid [jq/jaq syntax](https://jqlang.github.io/jq/manual/v1.6/).
#[mel_treatment(
    input json Stream<string>
    output parsed Stream<Vec<string>>
    output error Stream<Vec<string>>
    output failures Block<Vec<string>>
)]
pub async fn json_query(query: string) {

    let mut defs = ParseCtx::new(Vec::new());
    defs.insert_natives(jaq_core::core());
    defs.insert_defs(jaq_std::std());

    let (filter, errs) = jaq_parse::parse(&query, jaq_parse::main());
    if !errs.is_empty() {
        let _ = failures.send_one_vec_string(errs.into_iter().map(|e| e.to_string()).collect()).await;
    }
    else {
        let filter = defs.compile(filter.unwrap());
        if !defs.errs.is_empty() {
            let _ = failures.send_one_vec_string(defs.errs.into_iter().map(|e| e.0.to_string()).collect()).await;
        }
        else {
            while let Ok(json) = json.recv_one_string().await {

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
                        if let (Err(_), Err(_)) = (parsed.send_one_vec_string(outputs).await, error.send_one_vec_string(errors).await) {
                            break;
                        }
                    },
                    Err(err) => {
                        let _ = error.send_one_vec_string(vec![err.to_string()]).await;
                    },
                }

                
            }
        }
    }
}

mel_package!();