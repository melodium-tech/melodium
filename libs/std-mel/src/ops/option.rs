
use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

#[mel_function(
    generic T ()
)]
pub fn unwrap_or(option: Option<T>, default: T) -> T {
    option.unwrap_or(default)
}

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