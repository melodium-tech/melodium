use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_treatment};
use std::collections::HashMap;
use std::sync::Arc;

pub mod block;

/// A heterogeneous key→value map where keys are strings and values can be any Mélodium type.
///
/// Used to pass structured data between treatments and to build JSON-like payloads.
/// Supports `entry`, `get`, `insert`, and `merge` operations.
/// Later entries with the same key overwrite earlier ones.
#[mel_data(
    traits (PartialEquality Serialize Display)
)]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Map {
    pub map: HashMap<String, Value>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn new_with(map: HashMap<String, Value>) -> Self {
        Self { map }
    }
}

impl Display for Map {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{:#?}", self)
    }
}

/// Build a `Map` by merging a list of single-entry maps.
///
/// Each element in `entries` should be produced by `|entry(key, value)`.
/// Later entries with the same key overwrite earlier ones.
#[mel_function]
pub fn map(entries: Vec<Map>) -> Map {
    let mut map = HashMap::new();
    for submap in entries {
        map.extend(submap.map);
    }
    Map { map }
}

/// Build a single-entry `Map` mapping `key` to `value`.
///
/// Typically used as an argument to `|map([...])` to construct multi-entry maps.
#[mel_function(
    generic T ()
)]
pub fn entry(key: string, value: T) -> Map {
    let mut map = HashMap::new();
    map.insert(key, value);
    Map { map }
}

/// For every `value` received on the stream, produce a single-entry `Map` with `key` → `value` and emit it on `map`.
#[mel_treatment(
    generic T ()
    input value Stream<T>
    output map Stream<Map>
)]
pub async fn entry(key: string) {
    while let Ok(value) = value.recv_one().await {
        let mut new_map = HashMap::new();
        new_map.insert(key.clone(), value);
        let new_map = Map { map: new_map };
        check!(map.send_one(Value::Data(Arc::new(new_map))).await)
    }
}

/// Look up `key` in `map` and return its value as `Option<T>`, or `none` if the key is absent or the stored value is of a different type.
#[mel_function(
    generic T ()
)]
pub fn get(map: Map, key: string) -> Option<T> {
    generics
        .get("T")
        .map(|dt| map.map.get(&key).cloned().filter(|v| &v.datatype() == dt))
        .flatten()
}

/// For every `map` received on the stream, look up `key` and emit the result as `Option<T>` on `value`.
///
/// Emits `none` if the key is absent or the stored value does not match type `T`.
#[mel_treatment(
    generic T ()
    input map Stream<Map>
    output value Stream<Option<T>>
)]
pub async fn get(key: string) {
    while let Ok(map) = map.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Map>()
            .unwrap()
    }) {
        check!(value.send_one(map.map.get(&key).cloned().into()).await)
    }
}

/// Return a copy of `map` with `key` set to `value`, overwriting any existing entry for that key.
#[mel_function(
    generic T ()
)]
pub fn insert(mut map: Map, key: string, value: T) -> Map {
    map.map.insert(key, value);
    map
}

/// For every (`base`, `value`) pair received from the two streams, insert `key` → `value` into a copy of `base` and emit it on `map`.
#[mel_treatment(
    generic T ()
    input base Stream<Map>
    input value Stream<T>
    output map Stream<Map>
)]
pub async fn insert(key: string) {
    while let (Ok(base), Ok(value)) = (
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
        check!(map.send_one(Value::Data(Arc::new(new_map))).await)
    }
}
