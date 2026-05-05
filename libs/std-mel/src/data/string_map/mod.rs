use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_treatment};
use std::collections::HashMap;
use std::sync::Arc;

pub mod block;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A `string`→`string` map; serialisable and equatable.
///
/// Commonly used for environment variables and HTTP headers.
/// Supports `entry`, `get`, `insert`, and `merge` operations.
/// Later entries with the same key overwrite earlier ones.
#[mel_data(traits(Serialize Deserialize PartialEquality Equality))]
pub struct StringMap {
    pub map: HashMap<String, String>,
}

impl StringMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn new_with(map: HashMap<String, String>) -> Self {
        Self { map }
    }
}

impl Display for StringMap {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{:#?}", self)
    }
}

/// Build a `StringMap` by merging a list of single-entry maps.
///
/// Each element in `entries` should be produced by `|entry(key, value)`.
/// Later entries with the same key overwrite earlier ones.
#[mel_function]
pub fn map(entries: Vec<StringMap>) -> StringMap {
    let mut map = HashMap::new();
    for submap in entries {
        map.extend(submap.map);
    }
    StringMap { map }
}

/// Build a single-entry `StringMap` mapping `key` to `value`.
///
/// Typically used as an argument to `|map([...])` to construct multi-entry maps.
#[mel_function]
pub fn entry(key: string, value: string) -> StringMap {
    let mut map = HashMap::new();
    map.insert(key, value);
    StringMap { map }
}

/// For every `value` received on the stream, produce a single-entry `StringMap` with `key` → `value` and emit it on `map`.
#[mel_treatment(
    input value Stream<string>
    output map Stream<StringMap>
)]
pub async fn entry(key: string) {
    while let Ok(value) = value
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        let mut new_map = HashMap::new();
        new_map.insert(key.clone(), value);
        let new_map = StringMap { map: new_map };
        check!(map.send_one(Value::Data(Arc::new(new_map))).await)
    }
}

/// Look up `key` in `map` and return its value, or `none` if the key is absent.
#[mel_function]
pub fn get(map: StringMap, key: string) -> Option<string> {
    map.map.get(&key).cloned()
}

/// For every `map` received on the stream, look up `key` and emit the result as `Option<string>` on `value`.
#[mel_treatment(
    input map Stream<StringMap>
    output value Stream<Option<string>>
)]
pub async fn get(key: string) {
    while let Ok(map) = map.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<StringMap>()
            .unwrap()
    }) {
        check!(value.send_one(map.map.get(&key).cloned().into()).await)
    }
}

/// Return a copy of `map` with `key` set to `value`, overwriting any existing entry for that key.
#[mel_function]
pub fn insert(mut map: StringMap, key: string, value: string) -> StringMap {
    map.map.insert(key, value);
    map
}

/// For every (`base`, `value`) pair received from the two streams, insert `key` → `value` into a copy of `base` and emit it on `map`.
#[mel_treatment(
    input base Stream<StringMap>
    input value Stream<string>
    output map Stream<StringMap>
)]
pub async fn insert(key: string) {
    while let (Ok(base), Ok(value)) = (
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
        check!(map.send_one(Value::Data(Arc::new(new_map))).await)
    }
}
