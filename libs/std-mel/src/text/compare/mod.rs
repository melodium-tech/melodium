pub mod char;

use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Tells if strings exactly matches a pattern.
#[mel_treatment(
    input text Stream<string>
    output matches Stream<bool>
)]
pub async fn exact(pattern: string) {
    while let Ok(text) = text
        .recv_many()
        .await
        .map(|values: TransmissionValue| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        check!(
            matches
                .send_many(
                    text.into_iter()
                        .map(|txt| txt == pattern)
                        .collect::<VecDeque<_>>()
                        .into()
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
    while let Ok(text) = text
        .recv_many()
        .await
        .map(|values: TransmissionValue| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        check!(
            matches
                .send_many(
                    text.into_iter()
                        .map(|txt| txt.starts_with(&pattern))
                        .collect::<VecDeque<_>>()
                        .into()
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
    while let Ok(text) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        check!(
            matches
                .send_many(
                    text.into_iter()
                        .map(|txt| txt.ends_with(&pattern))
                        .collect::<VecDeque<_>>()
                        .into()
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
