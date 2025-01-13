use crate::data::*;
use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Rescale stream of strings.
///
/// _Rescaling_ means that strings sent throught stream are rearranged according to the `delimiter`.
///
/// Unscaled stream can basically be cut at any position:
/// ```
/// "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean qua"
/// "m velit, tristique et arcu in, viverra pulvinar ante. Interdum et m"
/// "alesuada fames ac ante ipsum primis in faucibus. Cras varius, augue"
/// " ac fringilla placerat, nibh lorem laoreet enim, sed fermentum libe"
/// " ro justo ut sapien."
/// ```
///
/// While treatments may expect well-defined strings:
/// ```
/// "Lorem ipsum dolor sit amet, consectetur adipiscing elit."
/// "Aenean quam velit, tristique et arcu in, viverra pulvinar ante."
/// "Interdum et malesuada fames ac ante ipsum primis in faucibus."
/// "Cras varius, augue ac fringilla placerat, nibh lorem laoreet enim, sed fermentum libero justo ut sapien."
/// ```
#[mel_treatment(
    default delimiter "\n"
    input unscaled Stream<string>
    output scaled Stream<string>
)]
pub async fn rescale(delimiter: string) {
    let mut previous = String::new();
    'main: while let Ok(input) = unscaled
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        let splits: Vec<&str> = input.split_inclusive(&delimiter).collect();
        for split in splits {
            previous.push_str(split);
            if previous.ends_with(&delimiter) {
                let sendable = previous;
                previous = String::new();
                check!('main, scaled.send_one(sendable.into()).await);
            }
        }
    }
    if !previous.is_empty() {
        let _ = scaled.send_one(previous.into()).await;
    }
}

/// Split strings with delimiter.
///
/// `text` is splitted according to `delimiter`, and streamed as `splitted` vector.
/// - `inclusive`: set if the delimiter must be kept at the end of splitted strings (if present).
///
/// ```mermaid
/// graph LR
///     T("split()")
///     B["🟦"] -->|vector| T
///     
///     T -->|value| O["［🟦 🟦 🟦］"]
///
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    default inclusive true
    input text Stream<string>
    output splitted Stream<Vec<string>>
)]
pub async fn split(delimiter: string, inclusive: bool) {
    while let Ok(input) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        let mut output = VecDeque::with_capacity(input.len());

        if inclusive {
            input.into_iter().for_each(|text| {
                output.push_back(Value::Vec(
                    text.split_inclusive(&delimiter)
                        .map(|s| s.to_string().into())
                        .collect(),
                ))
            });
        } else {
            input.into_iter().for_each(|text| {
                output.push_back(Value::Vec(
                    text.split(&delimiter)
                        .map(|s| s.to_string().into())
                        .collect(),
                ))
            });
        }

        check!(splitted.send_many(TransmissionValue::Other(output)).await);
    }
}

/// Split strings with delimiter.
///
/// `text` is splitted as `Vec<string>` according to `delimiter`.
/// - `inclusive`: set if the delimiter must be kept at the end of splitted strings (if present).
#[mel_function]
pub fn split(text: string, delimiter: string, inclusive: bool) -> Vec<string> {
    if inclusive {
        text.split_inclusive(&delimiter)
            .map(|s| s.to_string())
            .collect()
    } else {
        text.split(&delimiter).map(|s| s.to_string()).collect()
    }
}

/// Trim stream of strings.
///
/// Stream strings with leading and trailing whitespace removed.
/// _Whitespace_ is defined according to the terms of the Unicode Derived Core Property `White_Space`, which includes newlines.
#[mel_treatment(
    input text Stream<string>
    output trimmed Stream<string>
)]
pub async fn trim() {
    while let Ok(mut text) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        text.iter_mut().for_each(|t| *t = t.trim().to_string());

        check!(trimmed.send_many(text.into()).await);
    }
}

/// Trim string.
///
/// Return string with leading and trailing whitespace removed.
/// _Whitespace_ is defined according to the terms of the Unicode Derived Core Property `White_Space`, which includes newlines.
#[mel_function]
pub fn trim(text: string) -> string {
    text.trim().to_string()
}

/// Format string.
///
/// Return string formatted with given entries.
/// Format string is expected to contains braced placeholders, like: `"Hello {name}!"`.
///
/// If a formatting error happens, like missing key of incorrect format string, _none_ is returned.
///
/// ⚠️ All entries values must implements `ToString` trait, entries that does not will be ignored.
#[mel_function]
pub fn format(format: string, entries: Map) -> Option<string> {
    let format_map = entries
        .map
        .into_iter()
        .filter(|(_, val)| {
            val.datatype()
                .implements(&melodium_core::common::descriptor::DataTrait::ToString)
        })
        .map(|(name, val)| (name, melodium_core::DataTrait::to_string(&val)))
        .collect();
    strfmt::strfmt(&format, &format_map).ok()
}

/// Format stream.
///
/// Stream string formatted with given entries.
/// Format string is expected to contains braced placeholders, like: `"Hello {name}!"`.
///
/// If a formatting error happens, like missing key of incorrect format string, _none_ is sent.
///
/// ⚠️ All entries values must implements `ToString` trait, entries that does not will be ignored.
#[mel_treatment(
    input entries Stream<Map>
    output formatted Stream<Option<string>>
)]
pub async fn format(format: string) {
    while let Ok(maps) = entries
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<Value>>::try_into(values).unwrap())
    {
        let formatted_str = maps
            .into_iter()
            .map(|map| {
                GetData::<std::sync::Arc<dyn Data>>::try_data(map)
                    .unwrap()
                    .downcast_arc::<Map>()
                    .unwrap()
            })
            .map(|map| {
                map.map
                    .iter()
                    .filter(|(_, val)| {
                        val.datatype()
                            .implements(&melodium_core::common::descriptor::DataTrait::ToString)
                    })
                    .map(|(name, val)| (name.clone(), melodium_core::DataTrait::to_string(val)))
                    .collect::<std::collections::HashMap<String, String>>()
            })
            .map(|map| {
                Value::Option(
                    strfmt::strfmt(&format, &map)
                        .map(|formatted| Box::new(Value::String(formatted)))
                        .ok(),
                )
            })
            .collect::<VecDeque<_>>();

        check!(
            formatted
                .send_many(TransmissionValue::Other(formatted_str))
                .await
        );
    }
}
