use melodium_core::{executive::*, *};
use melodium_macro::mel_data;
use std::collections::HashMap;

#[mel_data(
    traits (PartialEquality Serialize Deserialize Display)
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Structure {
    inner: HashMap<String, Item>,
}

impl Display for Structure {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{:#?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Item {
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

    Vec(Vec<Item>),
    Option(Option<Box<Item>>),

    Structure(Box<Structure>),
}
