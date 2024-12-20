use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_treatment};
use std::collections::HashMap;
use std::sync::Arc;

pub mod block;

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

/// Create a map from entries
#[mel_function]
pub fn map(entries: Vec<Map>) -> Map {
    let mut map = HashMap::new();
    for submap in entries {
        map.extend(submap.map);
    }
    Map { map }
}

/// Create a map with one entry
#[mel_function(
    generic T ()
)]
pub fn entry(key: string, value: T) -> Map {
    let mut map = HashMap::new();
    map.insert(key, value);
    Map { map }
}

/// Create maps with one entry
///
/// For every `value` coming through the stream, send a mono-entry map.
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

/// Get a map entry
#[mel_function(
    generic T ()
)]
pub fn get(map: Map, key: string) -> Option<T> {
    generics
        .get("T")
        .map(|dt| map.map.get(&key).cloned().filter(|v| &v.datatype() == dt))
        .flatten()
}

/// Get a map entry
///
/// For every `map` coming through the stream, get the `key` entry.
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

/// Insert one entry in a map
#[mel_function(
    generic T ()
)]
pub fn insert(mut map: Map, key: string, value: T) -> Map {
    map.map.insert(key, value);
    map
}

/// Create maps with one entry
///
/// For every `value` coming through the stream, insert it into the `base` map.
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
