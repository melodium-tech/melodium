mod data;
mod traits;

use super::Data;
use crate::descriptor::DataType;
pub use data::GetData;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Value {
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

    Vec(Vec<Value>),
    Option(Option<Box<Value>>),

    Data(Arc<dyn Data>),
}

impl Value {
    pub fn datatype(&self) -> DataType {
        match self {
            Value::Void(_) => DataType::Void,

            Value::I8(_) => DataType::I8,
            Value::I16(_) => DataType::I16,
            Value::I32(_) => DataType::I32,
            Value::I64(_) => DataType::I64,
            Value::I128(_) => DataType::I128,

            Value::U8(_) => DataType::U8,
            Value::U16(_) => DataType::U16,
            Value::U32(_) => DataType::U32,
            Value::U64(_) => DataType::U64,
            Value::U128(_) => DataType::U128,

            Value::F32(_) => DataType::F32,
            Value::F64(_) => DataType::F64,

            Value::Bool(_) => DataType::Bool,
            Value::Byte(_) => DataType::Byte,
            Value::Char(_) => DataType::Char,
            Value::String(_) => DataType::String,

            Value::Option(val) => val
                .as_ref()
                .map(|val| DataType::Option(Box::new(val.datatype())))
                .unwrap_or(DataType::Undetermined),
            Value::Vec(val) => val
                .first()
                .map(|val| DataType::Vec(Box::new(val.datatype())))
                .unwrap_or(DataType::Undetermined),

            Value::Data(obj) => DataType::Data(obj.descriptor()),
        }
    }

    /// Casts to `T`, e.g. `value.try_data::<HttpStatus>()`. A thin forward to `GetData`,
    /// but as an inherent method with `T` on the *method* rather than the trait, so the
    /// turbofish goes where callers naturally reach for it and `GetData` doesn't need to
    /// be imported just to call this directly (outside a generic context where `T` is
    /// already fixed, e.g. `InputExt::recv_one_as`, this was previously only reachable via
    /// the more awkward `GetData::<T>::try_data(value)`).
    pub fn try_data<T>(self) -> Result<T, ()>
    where
        Self: GetData<T>,
    {
        GetData::<T>::try_data(self)
    }

    /// Rough memory footprint of this value, in bytes.
    ///
    /// Used to bound how much data a transmission buffer accumulates before flushing
    /// (see `melodium-engine`'s `Output`), so it favors being cheap to compute over being
    /// exact. Every `Value` occupies `size_of::<Value>()` inline regardless of variant
    /// (the enum is sized for its largest payload) plus whatever content it owns on the
    /// heap; `Data` has no cheap way to know its real size without serializing it, so a
    /// conservative fixed estimate stands in for it.
    pub fn estimated_size(&self) -> usize {
        const DATA_ESTIMATE: usize = 128;

        std::mem::size_of::<Value>()
            + match self {
                Value::String(value) => value.len(),
                Value::Vec(values) => values.iter().map(Value::estimated_size).sum(),
                Value::Option(Some(value)) => value.estimated_size(),
                Value::Data(_) => DATA_ESTIMATE,
                _ => 0,
            }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Void(l0), Self::Void(r0)) => l0 == r0,
            (Self::I8(l0), Self::I8(r0)) => l0 == r0,
            (Self::I16(l0), Self::I16(r0)) => l0 == r0,
            (Self::I32(l0), Self::I32(r0)) => l0 == r0,
            (Self::I64(l0), Self::I64(r0)) => l0 == r0,
            (Self::I128(l0), Self::I128(r0)) => l0 == r0,
            (Self::U8(l0), Self::U8(r0)) => l0 == r0,
            (Self::U16(l0), Self::U16(r0)) => l0 == r0,
            (Self::U32(l0), Self::U32(r0)) => l0 == r0,
            (Self::U64(l0), Self::U64(r0)) => l0 == r0,
            (Self::U128(l0), Self::U128(r0)) => l0 == r0,
            (Self::F32(l0), Self::F32(r0)) => l0 == r0,
            (Self::F64(l0), Self::F64(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Byte(l0), Self::Byte(r0)) => l0 == r0,
            (Self::Char(l0), Self::Char(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Vec(l0), Self::Vec(r0)) => l0 == r0,
            (Self::Option(l0), Self::Option(r0)) => l0 == r0,
            (Self::Data(l0), Self::Data(r0)) => {
                if l0.descriptor() == r0.descriptor() {
                    if l0
                        .descriptor()
                        .implements()
                        .contains(&crate::descriptor::DataTrait::PartialEquality)
                    {
                        l0.partial_equality_eq(other)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod estimated_size_tests {
    use super::Value;

    #[test]
    fn scalar_costs_only_the_enum_footprint() {
        let base = std::mem::size_of::<Value>();
        assert_eq!(Value::U64(42).estimated_size(), base);
        assert_eq!(Value::Void(()).estimated_size(), base);
    }

    #[test]
    fn string_costs_enum_footprint_plus_its_bytes() {
        let base = std::mem::size_of::<Value>();
        let text = "hello world".to_string();
        let expected = base + text.len();
        assert_eq!(Value::String(text).estimated_size(), expected);
    }

    #[test]
    fn vec_sums_enum_footprint_of_every_element() {
        let base = std::mem::size_of::<Value>();
        let vec = Value::Vec(vec![Value::Byte(1), Value::Byte(2), Value::Byte(3)]);
        // Outer Vec's own footprint, plus one full Value-sized slot per byte:
        // this is exactly the ~30x-per-byte blow-up a packed `Value::Bytes` would avoid.
        assert_eq!(vec.estimated_size(), base + 3 * base);
    }

    #[test]
    fn none_option_costs_only_the_enum_footprint() {
        let base = std::mem::size_of::<Value>();
        assert_eq!(Value::Option(None).estimated_size(), base);
    }

    #[test]
    fn some_option_adds_the_inner_values_size() {
        let base = std::mem::size_of::<Value>();
        let text = "abcdef".to_string();
        let inner_size = base + text.len();
        let value = Value::Option(Some(Box::new(Value::String(text))));
        assert_eq!(value.estimated_size(), base + inner_size);
    }
}
