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
