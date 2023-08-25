use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Tells if chars exactly matches a reference.
#[mel_treatment(
    input chars Stream<char>
    output matches Stream<bool>
)]
pub async fn exact(reference: char) {
    while let Ok(chars) = chars.recv_char().await {
        check!(
            matches
                .send_bool(chars.into_iter().map(|char| char == reference).collect())
                .await
        );
    }
}

/// Tells if char exactly matches a reference.
#[mel_function]
pub fn exact(char: char, reference: char) -> bool {
    char == reference
}

/// Tells if chars are alphabetic.
#[mel_treatment(
    input chars Stream<char>
    output is Stream<bool>
)]
pub async fn is_alphabetic() {
    while let Ok(chars) = chars.recv_char().await {
        check!(
            is.send_bool(chars.into_iter().map(|char| char.is_alphabetic()).collect())
                .await
        );
    }
}

/// Tells if char is alphabetic.
#[mel_function]
pub fn is_alphabetic(char: char) -> bool {
    char.is_alphabetic()
}

/// Tells if chars are alphanumeric.
#[mel_treatment(
    input chars Stream<char>
    output is Stream<bool>
)]
pub async fn is_alphanumeric() {
    while let Ok(chars) = chars.recv_char().await {
        check!(
            is.send_bool(
                chars
                    .into_iter()
                    .map(|char| char.is_alphanumeric())
                    .collect()
            )
            .await
        );
    }
}

/// Tells if char is alphanumeric.
#[mel_function]
pub fn is_alphanumeric(char: char) -> bool {
    char.is_alphanumeric()
}

/// Tells if chars are ascii.
#[mel_treatment(
    input chars Stream<char>
    output is Stream<bool>
)]
pub async fn is_ascii() {
    while let Ok(chars) = chars.recv_char().await {
        check!(
            is.send_bool(chars.into_iter().map(|char| char.is_ascii()).collect())
                .await
        );
    }
}

/// Tells if char is ascii.
#[mel_function]
pub fn is_ascii(char: char) -> bool {
    char.is_ascii()
}

/// Tells if chars are control.
#[mel_treatment(
    input chars Stream<char>
    output is Stream<bool>
)]
pub async fn is_control() {
    while let Ok(chars) = chars.recv_char().await {
        check!(
            is.send_bool(chars.into_iter().map(|char| char.is_control()).collect())
                .await
        );
    }
}

/// Tells if char is control.
#[mel_function]
pub fn is_control(char: char) -> bool {
    char.is_control()
}

/// Tells if chars are digit.
///
/// - `base`: must be between 0 and 36, if over `is` will only be `false`.
#[mel_treatment(
    input chars Stream<char>
    output is Stream<bool>
)]
pub async fn is_digit(base: u8) {
    while let Ok(chars) = chars.recv_char().await {
        if base <= 36 {
            check!(
                is.send_bool(
                    chars
                        .into_iter()
                        .map(|char| char.is_digit(base as u32))
                        .collect()
                )
                .await
            );
        } else {
            check!(is.send_bool(vec![false; chars.len()]).await);
        }
    }
}

/// Tells if char is digit.
///
/// - `base`: must be between 0 and 36, if over function will return `false` in any case.
#[mel_function]
pub fn is_digit(char: char, base: u8) -> bool {
    if base <= 36 {
        char.is_digit(base as u32)
    } else {
        false
    }
}

/// Tells if chars are lowercase.
#[mel_treatment(
    input chars Stream<char>
    output is Stream<bool>
)]
pub async fn is_lowercase() {
    while let Ok(chars) = chars.recv_char().await {
        check!(
            is.send_bool(chars.into_iter().map(|char| char.is_lowercase()).collect())
                .await
        );
    }
}

/// Tells if char is lowercase.
#[mel_function]
pub fn is_lowercase(char: char) -> bool {
    char.is_lowercase()
}

/// Tells if chars are uppercase.
#[mel_treatment(
    input chars Stream<char>
    output is Stream<bool>
)]
pub async fn is_uppercase() {
    while let Ok(chars) = chars.recv_char().await {
        check!(
            is.send_bool(chars.into_iter().map(|char| char.is_uppercase()).collect())
                .await
        );
    }
}

/// Tells if char is uppercase.
#[mel_function]
pub fn is_uppercase(char: char) -> bool {
    char.is_uppercase()
}

/// Tells if chars are whitespace.
#[mel_treatment(
    input chars Stream<char>
    output is Stream<bool>
)]
pub async fn is_whitespace() {
    while let Ok(chars) = chars.recv_char().await {
        check!(
            is.send_bool(chars.into_iter().map(|char| char.is_whitespace()).collect())
                .await
        );
    }
}

/// Tells if char is whitespace.
#[mel_function]
pub fn is_whitespace(char: char) -> bool {
    char.is_whitespace()
}
