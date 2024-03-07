#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::*;
use melodium_macro::{check, mel_function, mel_package, mel_treatment};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std_mel::data::*;

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
/// `found` contains the found strings (or _none_ if corresonding `text` input do not match).
/// `error` is emitted only if regex contains error.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_treatment(
    input text Stream<string>
    output found Stream<Option<string>>
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
                let mut vec_found = VecDeque::with_capacity(text.len());

                for text in text {
                    match regex.find(&text) {
                        Some(m) => {
                            vec_found.push_back(Some(m.as_str().to_string()).into());
                        }
                        None => {
                            vec_found.push_back(Value::Option(None));
                        }
                    }
                }

                check!(found.send_many(TransmissionValue::Other(vec_found)).await)
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
pub fn find(text: string, #[mel(content(regex))] regex: string) -> Option<string> {
    match Regex::new(&regex) {
        Ok(regex) => regex.find(&text).map(|m| m.as_str().to_string()),
        Err(_) => None,
    }
}

/// Captures groups of text according to a regex.
///
/// Every string coming through the `text` stream is passed through `regex`.
/// `captured` contains the **named** groups contents (or _none_ if group is not captured).
/// `error` is emitted only if regex contains error.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_treatment(
    input text Stream<string>
    output captured Stream<Option<Map>>
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
                let mut vec_captured = VecDeque::with_capacity(text.len());

                for text in text {
                    match regex.captures(&text) {
                        Some(captures) => {
                            let mut map_captured = HashMap::new();

                            for name in regex.capture_names() {
                                if let Some(name) = name {
                                    if let Some(cap) = captures.name(name) {
                                        map_captured.insert(
                                            name.to_string(),
                                            Value::String(cap.as_str().to_string()),
                                        );
                                    }
                                }
                            }

                            vec_captured.push_back(Value::Option(Some(Box::new(Value::Data(
                                Arc::new(Map::new_with(map_captured)),
                            )))));
                        }
                        None => {
                            vec_captured.push_back(Value::Option(None));
                        }
                    }
                }

                check!(
                    captured
                        .send_many(TransmissionValue::Other(vec_captured))
                        .await
                )
            }
        }
        Err(err) => {
            let _ = error.send_one(err.to_string().into()).await;
        }
    }
}

/// Captures groups of text according to a regex.
///
/// If match, return a `Map` containing the captured **named** groups.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_function]
pub fn capture(text: string, #[mel(content(regex))] regex: string) -> Option<Map> {
    match Regex::new(&regex) {
        Ok(regex) => match regex.captures(&text) {
            Some(captures) => {
                let mut map_captured = HashMap::new();

                for name in regex.capture_names() {
                    if let Some(name) = name {
                        if let Some(cap) = captures.name(name) {
                            map_captured
                                .insert(name.to_string(), Value::String(cap.as_str().to_string()));
                        }
                    }
                }

                Some(Map::new_with(map_captured))
            }
            None => None,
        },
        Err(_) => None,
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
/// Return string with replaced content, or _none_ if an error in regex occured.
///
/// The regex syntax is Unicode-aware. Please refer to [Regex Syntax](https://docs.rs/regex/latest/regex/index.html#syntax)
/// in documentation for full syntax description.
#[mel_function]
pub fn replace(
    text: string,
    #[mel(content(regex))] regex: string,
    replacer: string,
) -> Option<string> {
    match Regex::new(&regex) {
        Ok(regex) => Some(regex.replace(&text, &replacer).to_string()),
        Err(_) => None,
    }
}

mel_package!();
