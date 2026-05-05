use super::*;
use melodium_macro::mel_treatment;
use std::collections::HashMap;
use std::sync::Arc;

/// When `value` is received, produce a single-entry `StringMap` with `key` → `value` and emit it on `map`.
#[mel_treatment(
    input value Block<string>
    output map Block<StringMap>
)]
pub async fn entry(key: string) {
    if let Ok(value) = value
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        let mut new_map = HashMap::new();
        new_map.insert(key.clone(), value);
        let new_map = StringMap { map: new_map };
        let _ = map.send_one(Value::Data(Arc::new(new_map))).await;
    }
}

/// Receive one `StringMap` block and emit the value stored under `key` as `Option<string>` on `value`.
///
/// Emits `none` if the key is absent.
#[mel_treatment(
    input map Block<StringMap>
    output value Block<Option<string>>
)]
pub async fn get(key: string) {
    if let Ok(map) = map.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<StringMap>()
            .unwrap()
    }) {
        let _ = value.send_one(map.map.get(&key).cloned().into()).await;
    }
}

/// Receive one `base` map and one `value` block, insert `key` → `value` into a copy of `base`, and emit the updated map on `map`.
#[mel_treatment(
    input base Block<StringMap>
    input value Block<string>
    output map Block<StringMap>
)]
pub async fn insert(key: string) {
    if let (Ok(base), Ok(value)) = (
        base.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<StringMap>()
                .unwrap()
        }),
        value
            .recv_one()
            .await
            .map(|val| GetData::<String>::try_data(val).unwrap()),
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
    input base Block<StringMap>
    input entries Block<StringMap>
    output merged Block<StringMap>
)]
pub async fn merge() {
    if let Ok(base) = base.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<StringMap>()
            .unwrap()
    }) {
        if let Ok(entries) = entries.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<StringMap>()
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
