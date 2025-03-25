use futures::{pin_mut, select, FutureExt};
use melodium_core::*;
use melodium_macro::mel_treatment;

/// Unwrap option block
///
/// Unwrap blocking option, ignoring it if set to _none_.
#[mel_treatment(
    generic T ()
    input option Block<Option<T>>
    output value Block<T>
)]
pub async fn unwrap() {
    if let Ok(pos_val) = option.recv_one().await {
        match pos_val {
            Value::Option(Some(val)) => {
                let _ = value.send_one(*val).await;
            }
            _ => {}
        }
    }
}

/// Unwrap option block with default value
///
/// Unwrap blocking option, sending `default` value if option is _none_.
#[mel_treatment(
    generic T ()
    input option Block<Option<T>>
    output value Block<T>
)]
pub async fn unwrap_or(default: T) {
    if let Ok(pos_val) = option.recv_one().await {
        match pos_val {
            Value::Option(Some(val)) => {
                let _ = value.send_one(*val).await;
            }
            Value::Option(None) => {
                let _ = value.send_one(default).await;
            }
            _ => {}
        }
    }
}

/// Wrap a block value into option
///
/// Takes a blocking value and turn if into filled option.
#[mel_treatment(
    generic T ()
    input value Block<T>
    output option Block<Option<T>>
)]
pub async fn wrap() {
    if let Ok(val) = value.recv_one().await {
        let _ = option.send_one(Value::Option(Some(Box::new(val)))).await;
    }
}

/// Map an option
///
/// Takes a blocking option and either emit the contained value on `value` or `none` if empty.
///
/// ℹ️ This treatment aims to be used in conjuction with `reduce`.
#[mel_treatment(
    generic T ()
    input option Block<Option<T>>
    output none Block<void>
    output value Block<T>
)]
pub async fn map() {
    if let Ok(val) = option.recv_one().await {
        match val {
            Value::Option(Some(val)) => {
                let _ = value.send_one(*val).await;
            }
            Value::Option(None) => {
                let _ = none.send_one(().into()).await;
            }
            _ => unreachable!(),
        }
    }
}

/// Reduce an option
///
/// Takes either `value` or `none` and emit the corresponding `option`.
///
/// ℹ️ This treatment aims to be used in conjuction with `map`.
#[mel_treatment(
    generic T ()
    input value Block<T>
    input none Block<void>
    output option Block<Option<T>>
)]
pub async fn reduce() {
    let value_arrived = async { (&value).recv_one().await }.fuse();
    let none_arrived = async { (&none).recv_one().await }.fuse();

    pin_mut!(value_arrived, none_arrived);

    loop {
        select! {
            value = value_arrived => {
                if let Ok(value) = value {
                    let _ = option.send_one(Value::Option(Some(Box::new(value)))).await;
                    break;
                }
            },
            none = none_arrived => {
                if let Ok(_) = none {
                    let _ = option.send_one(Value::Option(None)).await;
                    break;
                }
            },
            complete => break,
        };
    }
}
