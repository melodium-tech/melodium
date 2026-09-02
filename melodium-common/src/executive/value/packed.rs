use super::data::GetData;
use super::Value;
use crate::descriptor::DataType;
use std::sync::Arc;

/// A contiguous, `Arc`-shared array of one primitive scalar type.
///
/// Backs `Value::Packed`: the packed counterpart of `Value::Vec(Vec<Value>)` for a
/// homogeneous scalar array. Storage is `size_of::<T>()` bytes per element instead of
/// one full `Value` enum instance (~24-32 bytes) per element, and the `Arc` gives cheap
/// fan-out sharing on top. See ticket #116.
#[derive(Clone, Debug, PartialEq)]
pub enum PackedArray {
    I8(Arc<[i8]>),
    I16(Arc<[i16]>),
    I32(Arc<[i32]>),
    I64(Arc<[i64]>),
    I128(Arc<[i128]>),

    U8(Arc<[u8]>),
    U16(Arc<[u16]>),
    U32(Arc<[u32]>),
    U64(Arc<[u64]>),
    U128(Arc<[u128]>),

    F32(Arc<[f32]>),
    F64(Arc<[f64]>),

    Bool(Arc<[bool]>),
    /// Distinct from `U8`, mirroring `Value::Byte` vs `Value::U8`: this is the packed
    /// form of the mel `byte` type specifically, not of generic 8-bit unsigned integers.
    Byte(Arc<[u8]>),
    Char(Arc<[char]>),
}

impl PackedArray {
    pub fn len(&self) -> usize {
        match self {
            PackedArray::I8(a) => a.len(),
            PackedArray::I16(a) => a.len(),
            PackedArray::I32(a) => a.len(),
            PackedArray::I64(a) => a.len(),
            PackedArray::I128(a) => a.len(),
            PackedArray::U8(a) => a.len(),
            PackedArray::U16(a) => a.len(),
            PackedArray::U32(a) => a.len(),
            PackedArray::U64(a) => a.len(),
            PackedArray::U128(a) => a.len(),
            PackedArray::F32(a) => a.len(),
            PackedArray::F64(a) => a.len(),
            PackedArray::Bool(a) => a.len(),
            PackedArray::Byte(a) => a.len(),
            PackedArray::Char(a) => a.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The logical element type, used by `Value::datatype()` to report `Vec(element)`
    /// regardless of whether the value happens to be packed or boxed — `Packed` is
    /// purely an internal representation choice, invisible to the mel type system.
    pub fn element_datatype(&self) -> DataType {
        match self {
            PackedArray::I8(_) => DataType::I8,
            PackedArray::I16(_) => DataType::I16,
            PackedArray::I32(_) => DataType::I32,
            PackedArray::I64(_) => DataType::I64,
            PackedArray::I128(_) => DataType::I128,
            PackedArray::U8(_) => DataType::U8,
            PackedArray::U16(_) => DataType::U16,
            PackedArray::U32(_) => DataType::U32,
            PackedArray::U64(_) => DataType::U64,
            PackedArray::U128(_) => DataType::U128,
            PackedArray::F32(_) => DataType::F32,
            PackedArray::F64(_) => DataType::F64,
            PackedArray::Bool(_) => DataType::Bool,
            PackedArray::Byte(_) => DataType::Byte,
            PackedArray::Char(_) => DataType::Char,
        }
    }

    /// Rough memory footprint of the array's content, in bytes: element count times
    /// element width, with no per-element `Value` overhead (see `Value::estimated_size`).
    pub fn estimated_size(&self) -> usize {
        match self {
            PackedArray::I8(a) => a.len() * std::mem::size_of::<i8>(),
            PackedArray::I16(a) => a.len() * std::mem::size_of::<i16>(),
            PackedArray::I32(a) => a.len() * std::mem::size_of::<i32>(),
            PackedArray::I64(a) => a.len() * std::mem::size_of::<i64>(),
            PackedArray::I128(a) => a.len() * std::mem::size_of::<i128>(),
            PackedArray::U8(a) => a.len() * std::mem::size_of::<u8>(),
            PackedArray::U16(a) => a.len() * std::mem::size_of::<u16>(),
            PackedArray::U32(a) => a.len() * std::mem::size_of::<u32>(),
            PackedArray::U64(a) => a.len() * std::mem::size_of::<u64>(),
            PackedArray::U128(a) => a.len() * std::mem::size_of::<u128>(),
            PackedArray::F32(a) => a.len() * std::mem::size_of::<f32>(),
            PackedArray::F64(a) => a.len() * std::mem::size_of::<f64>(),
            PackedArray::Bool(a) => a.len() * std::mem::size_of::<bool>(),
            PackedArray::Byte(a) => a.len(),
            PackedArray::Char(a) => a.len() * std::mem::size_of::<char>(),
        }
    }

    /// Expands into one `Value` per element. This is the correctness fallback used when
    /// something needs the fully-boxed representation (e.g. extracting through the
    /// generic `Vec<T>` path, see `GetData<Vec<T>> for Value`) — it reintroduces exactly
    /// the per-element overhead `Packed` exists to avoid, so callers that can use
    /// `GetData<Arc<[T]>>` directly should prefer that instead.
    pub fn into_values(self) -> Vec<Value> {
        match self {
            PackedArray::I8(a) => a.iter().copied().map(Value::I8).collect(),
            PackedArray::I16(a) => a.iter().copied().map(Value::I16).collect(),
            PackedArray::I32(a) => a.iter().copied().map(Value::I32).collect(),
            PackedArray::I64(a) => a.iter().copied().map(Value::I64).collect(),
            PackedArray::I128(a) => a.iter().copied().map(Value::I128).collect(),
            PackedArray::U8(a) => a.iter().copied().map(Value::U8).collect(),
            PackedArray::U16(a) => a.iter().copied().map(Value::U16).collect(),
            PackedArray::U32(a) => a.iter().copied().map(Value::U32).collect(),
            PackedArray::U64(a) => a.iter().copied().map(Value::U64).collect(),
            PackedArray::U128(a) => a.iter().copied().map(Value::U128).collect(),
            PackedArray::F32(a) => a.iter().copied().map(Value::F32).collect(),
            PackedArray::F64(a) => a.iter().copied().map(Value::F64).collect(),
            PackedArray::Bool(a) => a.iter().copied().map(Value::Bool).collect(),
            PackedArray::Byte(a) => a.iter().copied().map(Value::Byte).collect(),
            PackedArray::Char(a) => a.iter().copied().map(Value::Char).collect(),
        }
    }
}

impl From<PackedArray> for Value {
    fn from(value: PackedArray) -> Self {
        Value::Packed(value)
    }
}

impl GetData<PackedArray> for Value {
    fn try_data(self) -> Result<PackedArray, ()> {
        match self {
            Value::Packed(arr) => Ok(arr),
            _ => Err(()),
        }
    }
}

// `From<Vec<T>>`/`From<Arc<[T]>> for PackedArray` and the matching `GetData<Arc<[T]>> for
// Value` fast extractor follow the exact same shape for every scalar type, so they're
// generated rather than hand-duplicated — see ticket #120's precedent for
// `TransmissionValue`. `U8` is the one type kept hand-written below, for the same reason
// it is in `TransmissionValue`: `Byte` and `U8` are both `u8`-backed but semantically
// distinct (mel `byte` vs a generic 8-bit integer), so `From<Vec<u8>>` must stay
// unambiguous (always `U8`) while extraction accepts either.
macro_rules! packed_array_scalar_type {
    ($variant:ident, $ty:ty) => {
        impl From<Vec<$ty>> for PackedArray {
            fn from(value: Vec<$ty>) -> Self {
                PackedArray::$variant(value.into())
            }
        }

        impl From<Arc<[$ty]>> for PackedArray {
            fn from(value: Arc<[$ty]>) -> Self {
                PackedArray::$variant(value)
            }
        }

        impl GetData<Arc<[$ty]>> for Value {
            fn try_data(self) -> Result<Arc<[$ty]>, ()> {
                match self {
                    Value::Packed(PackedArray::$variant(arr)) => Ok(arr),
                    _ => Err(()),
                }
            }
        }
    };
}

packed_array_scalar_type!(I8, i8);
packed_array_scalar_type!(I16, i16);
packed_array_scalar_type!(I32, i32);
packed_array_scalar_type!(I64, i64);
packed_array_scalar_type!(I128, i128);
packed_array_scalar_type!(U16, u16);
packed_array_scalar_type!(U32, u32);
packed_array_scalar_type!(U64, u64);
packed_array_scalar_type!(U128, u128);
packed_array_scalar_type!(F32, f32);
packed_array_scalar_type!(F64, f64);
packed_array_scalar_type!(Bool, bool);
packed_array_scalar_type!(Char, char);

impl From<Vec<u8>> for PackedArray {
    fn from(value: Vec<u8>) -> Self {
        PackedArray::U8(value.into())
    }
}

impl From<Arc<[u8]>> for PackedArray {
    fn from(value: Arc<[u8]>) -> Self {
        PackedArray::U8(value)
    }
}

impl GetData<Arc<[u8]>> for Value {
    fn try_data(self) -> Result<Arc<[u8]>, ()> {
        match self {
            Value::Packed(PackedArray::U8(arr)) => Ok(arr),
            Value::Packed(PackedArray::Byte(arr)) => Ok(arr),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod packed_array_tests {
    use super::*;

    #[test]
    fn estimated_size_has_no_per_element_value_overhead() {
        let arr = PackedArray::U8(Arc::from(vec![0u8; 4096]));
        assert_eq!(arr.estimated_size(), 4096);

        let arr = PackedArray::F64(Arc::from(vec![0f64; 10]));
        assert_eq!(arr.estimated_size(), 10 * std::mem::size_of::<f64>());
    }

    #[test]
    fn value_packed_reports_the_same_datatype_as_value_vec() {
        let packed = Value::Packed(PackedArray::Byte(Arc::from(vec![1u8, 2, 3])));
        let boxed = Value::Vec(vec![Value::Byte(1), Value::Byte(2), Value::Byte(3)]);
        assert_eq!(packed.datatype(), boxed.datatype());
    }

    #[test]
    fn value_packed_estimated_size_has_no_per_element_value_overhead() {
        let base = std::mem::size_of::<Value>();
        let value = Value::Packed(PackedArray::Byte(Arc::from(vec![0u8; 4096])));
        // Contrast with `Value::Vec(Vec<Value::Byte>)`, which costs `base + n * base`
        // for the same content (see `estimated_size_tests::vec_sums_enum_footprint_of_every_element`).
        assert_eq!(value.estimated_size(), base + 4096);
    }

    #[test]
    fn arc_u8_extraction_accepts_both_u8_and_byte_variants() {
        let as_u8 = Value::Packed(PackedArray::U8(Arc::from(vec![1u8, 2, 3])));
        let as_byte = Value::Packed(PackedArray::Byte(Arc::from(vec![4u8, 5, 6])));

        let from_u8: Arc<[u8]> = as_u8.try_data().unwrap();
        let from_byte: Arc<[u8]> = as_byte.try_data().unwrap();

        assert_eq!(&*from_u8, &[1, 2, 3]);
        assert_eq!(&*from_byte, &[4, 5, 6]);
    }

    #[test]
    fn from_vec_u8_always_produces_u8_not_byte() {
        let packed: PackedArray = vec![1u8, 2, 3].into();
        assert!(matches!(packed, PackedArray::U8(_)));
    }

    #[test]
    fn mismatched_type_extraction_fails() {
        let value = Value::Packed(PackedArray::I64(Arc::from(vec![1i64])));
        let result: Result<Arc<[u8]>, ()> = value.try_data();
        assert!(result.is_err());
    }

    #[test]
    fn into_values_roundtrips_element_by_element() {
        let packed = PackedArray::Byte(Arc::from(vec![1u8, 2, 3]));
        assert_eq!(
            packed.into_values(),
            vec![Value::Byte(1), Value::Byte(2), Value::Byte(3)]
        );
    }
}
