#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::*;
use melodium_macro::{check, mel_function, mel_package, mel_treatment};
use regex::Regex;

/// Matches stream of strings against a regex.
///
/// Every string coming through the `text` stream is matched against `regex`.
/// `matches` tells if matching were found or not.
/// `error` is emitted only if regex contains error.
///
/// The regex engine is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_treatment(
    input text Stream<string>
    output matches Stream<bool>
    output error Block<string>
)]
pub async fn matches(#[mel(content(regex))] regex: string) {
    match Regex::new(&regex) {
        Ok(regex) => {
            error.close().await;

            while let Ok(text) = text
                .recv_many()
                .await
                .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
            {
                check!(
                    matches
                        .send_many(
                            text.into_iter()
                                .map(|txt| regex.is_match(&txt))
                                .collect::<VecDeque<_>>()
                                .into()
                        )
                        .await
                );
            }
        }
        Err(err) => {
            let _ = error.send_one(err.to_string().into()).await;
        }
    }
}

/// Matches a string against a regex.
///
/// `text` is matched against `regex`, returns wether the match were successful or not.
///
/// The regex engine is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_function]
pub fn matches(text: string, #[mel(content(regex))] regex: string) -> bool {
    match Regex::new(&regex) {
        Ok(regex) => regex.is_match(&text),
        Err(_) => false,
    }
}

/// Find in stream of strings according to a regex.
///
/// Every string coming through the `text` stream is looked up with `regex`.
/// `is_found` tells if something were found or not, `found` contains the found strings
/// (or empty string if corresonding `text` input do not match).
/// `error` is emitted only if regex contains error.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_treatment(
    input text Stream<string>
    output is_found Stream<bool>
    output found Stream<string>
    output error Block<string>
)]
pub async fn find(#[mel(content(regex))] regex: string) {
    match Regex::new(&regex) {
        Ok(regex) => {
            error.close().await;

            while let Ok(text) = text
                .recv_many()
                .await
                .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
            {
                let mut vec_is_found = Vec::with_capacity(text.len());
                let mut vec_found = Vec::with_capacity(text.len());

                for text in text {
                    match regex.find(&text) {
                        Some(m) => {
                            vec_is_found.push(true);
                            vec_found.push(m.as_str().to_string());
                        }
                        None => {
                            vec_is_found.push(false);
                            vec_found.push(String::default());
                        }
                    }
                }

                if let (Err(_), Err(_)) = futures::join!(
                    is_found.send_many(vec_is_found.into()),
                    found.send_many(vec_found.into())
                ) {
                    break;
                }
            }
        }
        Err(err) => {
            let _ = error.send_one(err.to_string().into()).await;
        }
    }
}

/// Find in string according to a regex.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_function]
pub fn find(text: string, #[mel(content(regex))] regex: string) -> string {
    match Regex::new(&regex) {
        Ok(regex) => regex
            .find(&text)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default(),
        Err(_) => String::default(),
    }
}

/// Captures groups of text according to a regex.
///
/// Every string coming through the `text` stream is passed through `regex`.
/// `is_captured` tells for each group if something were found or not, `captured`
/// contains the groups contents (or empty string if group is not captured).
/// `error` is emitted only if regex contains error.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_treatment(
    input text Stream<string>
    output captured Stream<Vec<string>>
    output is_captured Stream<Vec<bool>>
    output error Block<string>
)]
pub async fn capture(#[mel(content(regex))] regex: string) {
    match Regex::new(&regex) {
        Ok(regex) => {
            error.close().await;

            while let Ok(text) = text
                .recv_many()
                .await
                .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
            {
                let mut vec_captured = Vec::with_capacity(text.len());
                let mut vec_is_captured = Vec::with_capacity(text.len());

                for text in text {
                    match regex.captures(&text) {
                        Some(m) => {
                            let mut vec_capt = Vec::new();
                            let mut vec_is_capt = Vec::new();
                            let mut it = m.iter();
                            while let Some(capt) = it.next() {
                                match capt {
                                    Some(s) => {
                                        vec_capt.push(s.as_str().to_string().into());
                                        vec_is_capt.push(true.into());
                                    }
                                    None => {
                                        vec_capt.push(String::default().into());
                                        vec_is_capt.push(false.into());
                                    }
                                }
                            }
                            vec_captured.push(Value::Vec(vec_capt));
                            vec_is_captured.push(Value::Vec(vec_is_capt));
                        }
                        None => {
                            vec_captured.push(Value::Vec(Vec::new()));
                            vec_is_captured.push(Value::Vec(Vec::new()));
                        }
                    }
                }

                if let (Err(_), Err(_)) = futures::join!(
                    is_captured.send_many(TransmissionValue::Other(vec_is_captured.into())),
                    captured.send_many(TransmissionValue::Other(vec_captured.into()))
                ) {
                    break;
                }
            }
        }
        Err(err) => {
            let _ = error.send_one(err.to_string().into()).await;
        }
    }
}

/// Captures groups of text according to a regex.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_function]
pub fn capture(text: string, #[mel(content(regex))] regex: string) -> Vec<string> {
    match Regex::new(&regex) {
        Ok(regex) => match regex.captures(&text) {
            Some(capt) => capt
                .iter()
                .map(|c| c.map(|s| s.as_str().to_string()).unwrap_or_default())
                .collect(),
            None => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}

/// Captures named groups of text according to a regex.
///
/// Every string coming through the `text` stream is passed through `regex`.
/// `names` tells the group names, `is_captured` tells for each group if something were found or not, `captured`
/// contains the groups contents (or empty string if group is not captured).
/// `error` is emitted only if regex contains error.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_treatment(
    input text Stream<string>
    output captured Stream<Vec<string>>
    output is_captured Stream<Vec<bool>>
    output names Stream<Vec<string>>
    output error Block<string>
)]
pub async fn capture_named(#[mel(content(regex))] regex: string) {
    match Regex::new(&regex) {
        Ok(regex) => {
            error.close().await;

            let contained_names: Vec<String> = regex
                .capture_names()
                .map(|name| name.map(|n| n.to_string()).unwrap_or_default())
                .collect();

            while let Ok(text) = text
                .recv_many()
                .await
                .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
            {
                let mut vec_captured = Vec::with_capacity(text.len());
                let mut vec_is_captured = Vec::with_capacity(text.len());
                let vec_names = vec![
                    contained_names
                        .iter()
                        .map(|cn| Value::String(cn.clone()))
                        .collect::<Vec<_>>();
                    text.len()
                ];

                for text in text {
                    match regex.captures(&text) {
                        Some(m) => {
                            let mut vec_capt = Vec::new();
                            let mut vec_is_capt = Vec::new();

                            for name in &contained_names {
                                match m.name(name.as_str()) {
                                    Some(s) => {
                                        vec_capt.push(s.as_str().to_string().into());
                                        vec_is_capt.push(true.into());
                                    }
                                    None => {
                                        vec_capt.push(String::default().into());
                                        vec_is_capt.push(false.into());
                                    }
                                }
                            }
                            vec_captured.push(Value::Vec(vec_capt));
                            vec_is_captured.push(Value::Vec(vec_is_capt));
                        }
                        None => {
                            vec_captured.push(Value::Vec(Vec::new()));
                            vec_is_captured.push(Value::Vec(Vec::new()));
                        }
                    }
                }

                if let (Err(_), Err(_), Err(_)) = futures::join!(
                    is_captured.send_many(TransmissionValue::Other(vec_is_captured.into())),
                    captured.send_many(TransmissionValue::Other(vec_captured.into())),
                    names.send_many(TransmissionValue::Other(
                        vec_names.into_iter().map(|i| Value::Vec(i)).collect()
                    ))
                ) {
                    break;
                }
            }
        }
        Err(err) => {
            let _ = error.send_one(err.to_string().into()).await;
        }
    }
}

/// Replace text according to a regex.
///
/// Every string coming through the `text` stream is passed through `regex`,
/// and `replacer` is applied.
/// `error` is emitted only if regex contains error.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_treatment(
    input text Stream<string>
    output replaced Stream<string>
    output error Block<string>
)]
pub async fn replace(#[mel(content(regex))] regex: string, replacer: string) {
    match Regex::new(&regex) {
        Ok(regex) => {
            error.close().await;

            while let Ok(text) = text
                .recv_many()
                .await
                .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
            {
                let mut vec_replaced = Vec::with_capacity(text.len());

                for text in text {
                    vec_replaced.push(regex.replace(&text, &replacer).to_string());
                }

                check!(replaced.send_many(vec_replaced.into()).await);
            }
        }
        Err(err) => {
            let _ = error.send_one(err.to_string().into()).await;
        }
    }
}

/// Replace text according to a regex and replacer.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_function]
pub fn replace(text: string, #[mel(content(regex))] regex: string, replacer: string) -> string {
    match Regex::new(&regex) {
        Ok(regex) => regex.replace(&text, &replacer).to_string(),
        Err(_) => String::default(),
    }
}

mel_package!();
