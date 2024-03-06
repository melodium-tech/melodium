use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};
use std::collections::HashMap;

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

/// Insert one entry in a map
#[mel_function(
    generic T ()
)]
pub fn insert(mut map: Map, key: string, value: T) -> Map {
    map.map.insert(key, value);
    map
}

#[mel_data(
    traits (PartialEquality Serialize Deserialize Display)
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Structure {
    inner: HashMap<String, StructureItem>,
}

impl Display for Structure {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{:#?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StructureItem {
    Void(()),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    F32(f32),
    F64(f64),

    Bool(bool),
    Byte(u8),
    Char(char),
    String(String),

    Vec(Vec<StructureItem>),
    Option(Option<Box<StructureItem>>),

    Structure(Box<Structure>),
}
