//! Wiping of plaintext held in memory.
//!
//! Plaintext revealed from secrets, or held by inline secrets, is overwritten once it is not
//! needed anymore, and at the latest before the process exits. This is best effort: values
//! of custom `Data` types are only dropped, and copies made by reallocations, third-party
//! crates, child processes or the operating system are out of reach.

use crate::executive::{PackedArray, Value};
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use zeroize::Zeroize;

/// Holder of plaintext that can be wiped while still in use, such as a cache.
pub trait Wipe: Send + Sync {
    /// Overwrites the plaintext held, giving whether there was any.
    /// The plaintext is unavailable afterwards.
    fn wipe(&self) -> bool;
}

static HOLDERS: Mutex<Vec<Weak<dyn Wipe>>> = Mutex::new(Vec::new());
static WIPED: AtomicUsize = AtomicUsize::new(0);

/// Registers a holder, so that `wipe_all` wipes it if it is still alive.
pub fn register_wipe(holder: Weak<dyn Wipe>) {
    let mut holders = HOLDERS.lock().unwrap();
    holders.retain(|holder| holder.strong_count() > 0);
    holders.push(holder);
}

/// Counts a wiped plaintext holder, for `wiped_count`.
pub fn count_wiped() {
    WIPED.fetch_add(1, Ordering::Relaxed);
}

/// Gives how many plaintext holders were wiped since the process started.
pub fn wiped_count() -> usize {
    WIPED.load(Ordering::Relaxed)
}

/// Wipes every registered holder still alive, giving how many held plaintext.
///
/// To run right before the process exits. When the `MELODIUM_WIPE_TRACE` environment
/// variable is set, what was wiped is written to the standard error output.
pub fn wipe_all() -> usize {
    let holders = std::mem::take(&mut *HOLDERS.lock().unwrap());
    let mut wiped = 0;
    for holder in holders {
        if let Some(holder) = holder.upgrade() {
            if holder.wipe() {
                wiped += 1;
            }
        }
    }
    if std::env::var_os("MELODIUM_WIPE_TRACE").is_some() {
        eprintln!(
            "wipe: {wiped} plaintext holders wiped before exit, {} since start",
            wiped_count()
        );
    }
    wiped
}

/// Overwrites a value and leaves it empty, as far as possible.
///
/// Packed arrays shared with other values cannot be overwritten, and custom `Data`
/// values are only dropped.
pub fn wipe_value(value: &mut Value) {
    fn wipe_packed<T: Zeroize>(values: &mut Arc<Vec<T>>) {
        if let Some(values) = Arc::get_mut(values) {
            values.zeroize();
        }
    }

    match value {
        Value::String(text) => text.zeroize(),
        Value::Vec(values) => {
            values.iter_mut().for_each(wipe_value);
            values.clear();
        }
        Value::Option(Some(inner)) => wipe_value(inner),
        Value::Packed(packed) => match packed {
            PackedArray::I8(values) => wipe_packed(values),
            PackedArray::I16(values) => wipe_packed(values),
            PackedArray::I32(values) => wipe_packed(values),
            PackedArray::I64(values) => wipe_packed(values),
            PackedArray::I128(values) => wipe_packed(values),
            PackedArray::U8(values) | PackedArray::Byte(values) => wipe_packed(values),
            PackedArray::U16(values) => wipe_packed(values),
            PackedArray::U32(values) => wipe_packed(values),
            PackedArray::U64(values) => wipe_packed(values),
            PackedArray::U128(values) => wipe_packed(values),
            PackedArray::F32(values) => wipe_packed(values),
            PackedArray::F64(values) => wipe_packed(values),
            PackedArray::Bool(values) => wipe_packed(values),
            PackedArray::Char(values) => wipe_packed(values),
        },
        Value::I8(number) => number.zeroize(),
        Value::I16(number) => number.zeroize(),
        Value::I32(number) => number.zeroize(),
        Value::I64(number) => number.zeroize(),
        Value::I128(number) => number.zeroize(),
        Value::U8(number) | Value::Byte(number) => number.zeroize(),
        Value::U16(number) => number.zeroize(),
        Value::U32(number) => number.zeroize(),
        Value::U64(number) => number.zeroize(),
        Value::U128(number) => number.zeroize(),
        Value::F32(number) => number.zeroize(),
        Value::F64(number) => number.zeroize(),
        Value::Bool(boolean) => boolean.zeroize(),
        Value::Char(character) => character.zeroize(),
        Value::Void(_) | Value::Option(None) | Value::Secret(_) | Value::Data(_) => {}
    }
    *value = Value::Void(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_overwritten_and_emptied() {
        let mut value = Value::Vec(vec![
            Value::String("wipe-sentinel".to_string()),
            Value::Option(Some(Box::new(Value::U64(42)))),
        ]);
        wipe_value(&mut value);
        assert_eq!(value, Value::Void(()));

        let bytes = Arc::new(vec![1u8, 2, 3]);
        let mut unique = Value::Packed(PackedArray::Byte(Arc::clone(&bytes)));
        drop(bytes);
        wipe_value(&mut unique);
        assert_eq!(unique, Value::Void(()));

        // A shared array is left to its other users.
        let shared = Arc::new(vec![1u8, 2, 3]);
        let mut value = Value::Packed(PackedArray::Byte(Arc::clone(&shared)));
        wipe_value(&mut value);
        assert_eq!(*shared, vec![1u8, 2, 3]);
    }
}
