use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

pub mod block;

/// Unwrap option of return default value
#[mel_function(
    generic T ()
)]
pub fn unwrap_or(option: Option<T>, default: T) -> T {
    option.unwrap_or(default)
}

/// Unwrap option stream
///
/// Unwrap every option in stream, ignoring values set to _none_.
#[mel_treatment(
    generic T ()
    input option Stream<Option<T>>
    output value Stream<T>
)]
pub async fn unwrap() {
    while let Ok(values) = option
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            value
                .send_many(TransmissionValue::Other(
                    values
                        .into_iter()
                        .filter_map(|val| match val {
                            Value::Option(Some(val)) => Some(*val),
                            _ => None,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Unwrap option stream with default value
///
/// Unwrap every option in stream, sending `default` value when option is _none_.
#[mel_treatment(
    generic T ()
    input option Stream<Option<T>>
    output value Stream<T>
)]
pub async fn unwrap_or(default: T) {
    while let Ok(values) = option
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            value
                .send_many(TransmissionValue::Other(
                    values
                        .into_iter()
                        .filter_map(|val| match val {
                            Value::Option(Some(val)) => Some(*val),
                            Value::Option(None) => Some(default.clone()),
                            _ => None,
                        })
                        .collect()
                ))
                .await
        )
    }
}

/// Unwrap option stream until _none_ is present
///
/// Unwrap every option in stream, and stop at the first _none_ value encountered.
#[mel_treatment(
    generic T ()
    input option Stream<Option<T>>
    output value Stream<T>
)]
pub async fn fuse() {
    'main: while let Ok(values) = option
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        for val in values {
            match val {
                Value::Option(Some(val)) => check!('main, value.send_one(*val).await),
                _ => break 'main,
            }
        }
    }
}

/// Wrap a value in an option
#[mel_function(
    generic T ()
)]
pub fn wrap(value: T) -> Option<T> {
    Some(value)
}

/// Wrap values in options
///
/// Takes a stream of values and turn them into filled options.
#[mel_treatment(
    generic T ()
    input value Stream<T>
    output option Stream<Option<T>>
)]
pub async fn wrap() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            option
                .send_many(TransmissionValue::Other(
                    values
                        .into_iter()
                        .map(|val| Value::Option(Some(Box::new(val))))
                        .collect()
                ))
                .await
        )
    }
}
