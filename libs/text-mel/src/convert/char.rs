use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Convert stream of chars into stream of strings.
#[mel_treatment(
    input chars Stream<char>
    output text Stream<string>
)]
pub async fn to_string() {
    while let Ok(chars) = chars.recv_char().await {

        check!(text.send_one_string(chars.into_iter().collect()).await);
    }
}

/// Convert stream of string into stream of chars.
#[mel_treatment(
    input text Stream<string>
    output chars Stream<char>
)]
pub async fn from_string() {
    while let Ok(text) = text.recv_string().await {

        let mut output = Vec::new();
        for text in text {
            output.extend(text.chars());
        }

        check!(chars.send_char(output).await);
    }
}

/// Converts stream of chars into UTF-8 encoded stream of bytes.
#[mel_treatment(
    input text Stream<char>
    output encoded Stream<byte>
)]
pub async fn to_utf8() {
    while let Ok(text) = text.recv_char().await {

        let mut output = Vec::new();
        for text in text {
            output.extend(text.to_string().as_bytes());
        }

        check!(encoded.send_byte(output).await);
    }
}

/// Convert char into UTF-8 encoded vector of bytes.
#[mel_function]
pub fn to_utf8(char: char) -> Vec<byte> {
    char.to_string().as_bytes().into()
}

/// Converts vector of bytes into vector of char according to UTF-8 encoding.
/// 
/// If any sequence of bytes doesn't follow UTF-8 encoding, it is replaced by the `U+FFFD REPLACEMENT CHARACTER` (�).
#[mel_function]
pub fn from_utf8(encoded: Vec<byte>) -> Vec<char> {
    String::from_utf8_lossy(&encoded).chars().collect()
}
