pub mod char;

use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Return `true` if `text` contains `substring`.
#[mel_function]
pub fn contains(text: string, substring: string) -> bool {
    text.contains(&substring)
}

/// For each string in `text`, emit `true` through `contains` if it contains `substring`.
#[mel_treatment(
    input text Stream<string>
    output contains Stream<bool>
)]
pub async fn contains(substring: string) {
    while let Ok(values) = text.recv_many_as::<string>().await {
        check!(
            contains
                .send_many_as(
                    values
                        .iter()
                        .map(|val| val.contains(&substring))
                        .collect::<Vec<bool>>()
                )
                .await
        )
    }
}

/// Tells if strings exactly matches a pattern.
#[mel_treatment(
    input text Stream<string>
    output matches Stream<bool>
)]
pub async fn exact(pattern: string) {
    while let Ok(text) = text.recv_many_as::<string>().await {
        check!(
            matches
                .send_many_as(
                    text.into_iter()
                        .map(|txt| txt == pattern)
                        .collect::<Vec<_>>()
                )
                .await
        );
    }
}

/// Tells if string exactly matches a pattern.
#[mel_function]
pub fn exact(text: string, pattern: string) -> bool {
    text == pattern
}

/// Tells if strings starts with a pattern.
#[mel_treatment(
    input text Stream<string>
    output matches Stream<bool>
)]
pub async fn starts_with(pattern: string) {
    while let Ok(text) = text.recv_many_as::<string>().await {
        check!(
            matches
                .send_many_as(
                    text.into_iter()
                        .map(|txt| txt.starts_with(&pattern))
                        .collect::<Vec<_>>()
                )
                .await
        );
    }
}

/// Tells if string starts with a pattern.
#[mel_function]
pub fn starts_with(text: string, pattern: string) -> bool {
    text.starts_with(&pattern)
}

/// Tells if strings ends with a pattern.
#[mel_treatment(
    input text Stream<string>
    output matches Stream<bool>
)]
pub async fn ends_with(pattern: string) {
    while let Ok(text) = text.recv_many_as::<string>().await {
        check!(
            matches
                .send_many_as(
                    text.into_iter()
                        .map(|txt| txt.ends_with(&pattern))
                        .collect::<Vec<_>>()
                )
                .await
        );
    }
}

/// Tells if string ends with a pattern.
#[mel_function]
pub fn ends_with(text: string, pattern: string) -> bool {
    text.ends_with(&pattern)
}
