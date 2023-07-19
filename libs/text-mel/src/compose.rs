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
/// While treamtments may expect well-defined strings:
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
    'main: while let Ok(input) = unscaled.recv_one_string().await {
        let splits: Vec<&str> = input.split_inclusive(&delimiter).collect();
        for split in splits {
            previous.push_str(split);
            if previous.ends_with(&delimiter) {
                check!('main, scaled.send_one_string(previous).await);
                previous = String::new();
            }
        }
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
    while let Ok(input) = text.recv_string().await {
        let mut output = Vec::with_capacity(input.len());

        if inclusive {
            input.into_iter().for_each(|text| {
                output.push(
                    text.split_inclusive(&delimiter)
                        .map(|s| s.to_string())
                        .collect(),
                )
            });
        } else {
            input.into_iter().for_each(|text| {
                output.push(text.split(&delimiter).map(|s| s.to_string()).collect())
            });
        }

        check!(splitted.send_vec_string(output).await);
    }
}

/// Split strings with delimiter.
///
/// `text` is splitted as `Vec<string>` according to `delimiter`.
/// - `inclusive`: set if the delimiter must be kept at the end of splitted strings (if present).
#[mel_function(
    default inclusive true
)]
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
    while let Ok(mut text) = text.recv_string().await {
        text.iter_mut().for_each(|t| *t = t.trim().to_string());

        check!(trimmed.send_string(text).await);
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
