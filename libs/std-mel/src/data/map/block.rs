use super::*;
use melodium_macro::mel_treatment;
use std::collections::HashMap;
use std::sync::Arc;

/// When `value` is received, produce a single-entry `Map` with `key` → `value` and emit it on `map`.
#[mel_treatment(
    generic T ()
    input value Block<T>
    output map Block<Map>
)]
pub async fn entry(key: string) {
    if let Ok(value) = value.recv_one().await {
        let mut new_map = HashMap::new();
        new_map.insert(key.clone(), value);
        let new_map = Map { map: new_map };
        let _ = map.send_one(Value::Data(Arc::new(new_map))).await;
    }
}

/// Receive one `Map` block and emit the value stored under `key` as `Option<T>` on `value`.
///
/// Emits `none` if the key is absent or the stored value does not match type `T`.
#[mel_treatment(
    generic T ()
    input map Block<Map>
    output value Block<Option<T>>
)]
pub async fn get(key: string) {
    if let Ok(map) = map.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Map>()
            .unwrap()
    }) {
        let _ = value.send_one(map.map.get(&key).cloned().into()).await;
    }
}

/// Receive one `base` map and one `value` block, insert `key` → `value` into a copy of `base`, and emit the updated map on `map`.
#[mel_treatment(
    generic T ()
    input base Block<Map>
    input value Block<T>
    output map Block<Map>
)]
pub async fn insert(key: string) {
    if let (Ok(base), Ok(value)) = (
        base.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Map>()
                .unwrap()
        }),
        value.recv_one().await,
    ) {
        let mut new_map = Arc::unwrap_or_clone(base);
        new_map.map.insert(key.clone(), value);
        let _ = map.send_one(Value::Data(Arc::new(new_map))).await;
    }
}

/// Merge two maps
///
/// Merge map `entries` in `base`.
/// `entries` erase existing entries in `base` if they already exists.
/// `entries` can be omitted (closed input) and `merge` will still be emitted if `base` is received.
#[mel_treatment(
    input base Block<Map>
    input entries Block<Map>
    output merged Block<Map>
)]
pub async fn merge() {
    if let Ok(base) = base.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Map>()
            .unwrap()
    }) {
        if let Ok(entries) = entries.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Map>()
                .unwrap()
        }) {
            let mut new_map = Arc::unwrap_or_clone(base);
            for (key, value) in &entries.map {
                new_map.map.insert(key.clone(), value.clone());
            }

            let _ = merged.send_one(Value::Data(Arc::new(new_map))).await;
        } else {
            let _ = merged.send_one(Value::Data(base)).await;
        }
    }
}
