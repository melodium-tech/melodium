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
    while let Ok(text) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        let output = text
            .into_iter()
            .map(|text| Value::Vec(text.chars().map(|c| Value::Char(c)).collect()))
            .collect::<VecDeque<_>>();

        check!(chars.send_many(TransmissionValue::Other(output)).await);
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
    while let Ok(chars) = chars
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        let output = chars
            .into_iter()
            .map(|text| match text {
                Value::Vec(text) => text
                    .into_iter()
                    .map(|c| match c {
                        Value::Char(c) => c,
                        _ => panic!("char expected"),
                    })
                    .collect::<String>(),
                _ => panic!("Vec<char> expected"),
            })
            .collect::<VecDeque<_>>();

        check!(text.send_many(output.into()).await);
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
    while let Ok(text) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        let mut output = VecDeque::new();
        for text in text {
            output.extend(text.as_bytes());
        }

        check!(encoded.send_many(TransmissionValue::Byte(output)).await);
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
    while let Ok(encoded) = encoded
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<byte>>::try_into(values).unwrap())
    {
        let output = String::from_utf8_lossy(&encoded).to_string();

        check!(text.send_one(output.into()).await);
    }
}

/// Converts vector of bytes into a string according to UTF-8 encoding.
///
/// If any sequence of bytes doesn't follow UTF-8 encoding, it is replaced by the `U+FFFD REPLACEMENT CHARACTER` (�).
#[mel_function]
pub fn from_utf8(encoded: Vec<byte>) -> string {
    String::from_utf8_lossy(&encoded).to_string()
}
