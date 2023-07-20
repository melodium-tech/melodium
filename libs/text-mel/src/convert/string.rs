use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Convert stream of string into chars.
///
/// Each string is turned into equivalent vector of chars.
#[mel_treatment(
    input text Stream<string>
    output chars Stream<Vec<char>>
)]
pub async fn to_char() {
    while let Ok(text) = text.recv_string().await {
        let output = text
            .into_iter()
            .map(|text| text.chars().collect())
            .collect();

        check!(chars.send_vec_char(output).await);
    }
}

/// Convert string into vector of chars.
#[mel_function]
pub fn to_char(text: string) -> Vec<char> {
    text.chars().collect()
}

/// Convert stream of char vectors into stream of strings.
///
/// Each streamed char vector is turned into its string equivalent.
#[mel_treatment(
    input chars Stream<Vec<char>>
    output text Stream<string>
)]
pub async fn from_char() {
    while let Ok(chars) = chars.recv_vec_char().await {
        let output = chars
            .into_iter()
            .map(|text| text.into_iter().collect())
            .collect();

        check!(text.send_string(output).await);
    }
}

/// Convert vector of chars into string.
#[mel_function]
pub fn from_char(chars: Vec<char>) -> string {
    chars.into_iter().collect()
}

/// Converts stream of strings into UTF-8 encoded stream of bytes.
///
///
#[mel_treatment(
    input text Stream<string>
    output encoded Stream<byte>
)]
pub async fn to_utf8() {
    while let Ok(text) = text.recv_string().await {
        let mut output = Vec::new();
        for text in text {
            output.extend(text.as_bytes());
        }

        check!(encoded.send_byte(output).await);
    }
}

/// Convert string into UTF-8 encoded vector of bytes.
#[mel_function]
pub fn to_utf8(text: string) -> Vec<byte> {
    text.as_bytes().into()
}

/// Converts stream of bytes into stream of strings according to UTF-8 encoding.
///
/// If any sequence of bytes doesn't follow UTF-8 encoding, it is replaced by the `U+FFFD REPLACEMENT CHARACTER` (�).
#[mel_treatment(
    input encoded Stream<byte>
    output text Stream<string>
)]
pub async fn from_utf8() {
    while let Ok(encoded) = encoded.recv_byte().await {
        let output = String::from_utf8_lossy(&encoded).to_string();

        check!(text.send_one_string(output).await);
    }
}

/// Converts vector of bytes into a string according to UTF-8 encoding.
///
/// If any sequence of bytes doesn't follow UTF-8 encoding, it is replaced by the `U+FFFD REPLACEMENT CHARACTER` (�).
#[mel_function]
pub fn from_utf8(encoded: Vec<byte>) -> string {
    String::from_utf8_lossy(&encoded).to_string()
}
