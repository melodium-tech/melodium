use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_treatment};
use std::collections::HashMap;
use std::sync::Arc;

pub mod block;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Create a map from entries
#[mel_function]
pub fn map(entries: Vec<StringMap>) -> StringMap {
    let mut map = HashMap::new();
    for submap in entries {
        map.extend(submap.map);
    }
    StringMap { map }
}

/// Create a map with one entry
#[mel_function]
pub fn entry(key: string, value: string) -> StringMap {
    let mut map = HashMap::new();
    map.insert(key, value);
    StringMap { map }
}

/// Create maps with one entry
///
/// For every `value` coming through the stream, send a mono-entry map.
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

/// Get a map entry
#[mel_function]
pub fn get(map: StringMap, key: string) -> Option<string> {
    map.map.get(&key).cloned()
}

/// Get a map entry
///
/// For every `map` coming through the stream, get the `key` entry.
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

/// Insert one entry in a map
#[mel_function]
pub fn insert(mut map: StringMap, key: string, value: string) -> StringMap {
    map.map.insert(key, value);
    map
}

/// Insert entry in map
///
/// For every `value` coming through the stream, insert it into the `base` map.
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
