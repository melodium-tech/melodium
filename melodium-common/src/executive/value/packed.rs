use super::data::GetData;
use super::Value;
use crate::descriptor::DataType;
use std::any::Any;
use std::sync::Arc;

/// A contiguous, `Arc`-shared array of one primitive scalar type.
///
/// Backs `Value::Packed`: the packed counterpart of `Value::Vec(Vec<Value>)` for a
/// homogeneous scalar array. Storage is `size_of::<T>()` bytes per element instead of
/// one full `Value` enum instance (~24-32 bytes) per element.
///
/// Wraps `Arc<Vec<T>>` rather than `Arc<[T]>`: converting an existing `Vec<T>` into
/// `Arc<Vec<T>>` (`Arc::new`) never touches the element data — only a small new
/// allocation for the refcount header, with the `Vec`'s own `{ptr,len,cap}` moved into
/// it. `Arc<[T]>` looks similar but is a dynamically-sized type that must sit directly
/// beside its refcount header in one allocation, so `Arc<[T]>::from(vec)` always
/// reallocates and copies every element (verified empirically). The trade is that a
/// `Vec` built with spare capacity (e.g. via repeated `push`) keeps that slack for the
/// `Arc`'s lifetime instead of being trimmed — bounded (typically ≤2x from geometric
/// growth) and dwarfed by the per-element `Value` overhead this type avoids in the
/// first place. See ticket #116.
#[derive(Clone, Debug, PartialEq)]
pub enum PackedArray {
    I8(Arc<Vec<i8>>),
    I16(Arc<Vec<i16>>),
    I32(Arc<Vec<i32>>),
    I64(Arc<Vec<i64>>),
    I128(Arc<Vec<i128>>),

    U8(Arc<Vec<u8>>),
    U16(Arc<Vec<u16>>),
    U32(Arc<Vec<u32>>),
    U64(Arc<Vec<u64>>),
    U128(Arc<Vec<u128>>),

    F32(Arc<Vec<f32>>),
    F64(Arc<Vec<f64>>),

    Bool(Arc<Vec<bool>>),
    /// Distinct from `U8`, mirroring `Value::Byte` vs `Value::U8`: this is the packed
    /// form of the mel `byte` type specifically, not of generic 8-bit unsigned integers.
    Byte(Arc<Vec<u8>>),
    Char(Arc<Vec<char>>),
}

/// Reusable single-shot `Any` downcast: `Ok` if the caller's `T` is a real match for the
/// runtime type `U`, `Err` with the input handed back unchanged otherwise. Shared by both
/// `try_from_vec` and `try_into_vec` so the "probe a chain of concrete candidate types"
/// pattern is written once.
fn downcast_vec<T: 'static, U: 'static>(value: Vec<U>) -> Result<Vec<T>, Vec<U>> {
    let boxed: Box<dyn Any> = Box::new(value);
    match boxed.downcast::<Vec<T>>() {
        Ok(value) => Ok(*value),
        Err(boxed) => Err(*boxed.downcast::<Vec<U>>().unwrap()),
    }
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

    /// Expands into one `Value` per element. This is the correctness fallback for
    /// contexts that genuinely need the fully-boxed representation (e.g. `Display`) — it
    /// reintroduces exactly the per-element overhead `Packed` exists to avoid. Prefer
    /// `try_into_vec`/`GetData<Arc<Vec<T>>>` wherever the target type is known.
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

    /// Attempts to pack a `Vec<T>` using only `T: 'static` — no trait bound naming the
    /// packable types is needed, since the check happens at runtime via `Any`. Returns
    /// the vec back unchanged if `T` isn't one of the packable primitives, so the caller
    /// can fall back to the ordinary boxed representation. Never produces `Byte`: a
    /// `Vec<u8>` is ambiguous between "bytes" and "generic small integers", so this
    /// always resolves it to `U8`, exactly like `Value::from(u8)` never producing
    /// `Value::Byte` on its own.
    pub fn try_from_vec<T: 'static>(value: Vec<T>) -> Result<PackedArray, Vec<T>> {
        let value = match downcast_vec::<i8, T>(value) {
            Ok(v) => return Ok(PackedArray::I8(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<i16, T>(value) {
            Ok(v) => return Ok(PackedArray::I16(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<i32, T>(value) {
            Ok(v) => return Ok(PackedArray::I32(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<i64, T>(value) {
            Ok(v) => return Ok(PackedArray::I64(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<i128, T>(value) {
            Ok(v) => return Ok(PackedArray::I128(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<u8, T>(value) {
            Ok(v) => return Ok(PackedArray::U8(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<u16, T>(value) {
            Ok(v) => return Ok(PackedArray::U16(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<u32, T>(value) {
            Ok(v) => return Ok(PackedArray::U32(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<u64, T>(value) {
            Ok(v) => return Ok(PackedArray::U64(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<u128, T>(value) {
            Ok(v) => return Ok(PackedArray::U128(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<f32, T>(value) {
            Ok(v) => return Ok(PackedArray::F32(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<f64, T>(value) {
            Ok(v) => return Ok(PackedArray::F64(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<bool, T>(value) {
            Ok(v) => return Ok(PackedArray::Bool(Arc::new(v))),
            Err(v) => v,
        };
        let value = match downcast_vec::<char, T>(value) {
            Ok(v) => return Ok(PackedArray::Char(Arc::new(v))),
            Err(v) => v,
        };
        Err(value)
    }

    /// Attempts to extract a `Vec<T>` directly, with no `Value` ever created along the
    /// way — the fast counterpart to `into_values()` followed by per-element
    /// `GetData::<T>::try_data`. If the array's element type doesn't already match `T`
    /// exactly (via `Any`), the original `PackedArray` is handed back unchanged.
    ///
    /// Only makes one copy: `Arc::try_unwrap` succeeds for free when this is the sole
    /// reference to the array (the common case for a value that just arrived off a
    /// channel); it falls back to cloning only when the array is still shared elsewhere.
    pub fn try_into_vec<T: 'static>(self) -> Result<Vec<T>, PackedArray> {
        fn owned<U: Clone>(arc: Arc<Vec<U>>) -> Vec<U> {
            Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone())
        }

        match self {
            PackedArray::I8(a) => {
                downcast_vec::<T, i8>(owned(a)).map_err(|v| PackedArray::I8(Arc::new(v)))
            }
            PackedArray::I16(a) => {
                downcast_vec::<T, i16>(owned(a)).map_err(|v| PackedArray::I16(Arc::new(v)))
            }
            PackedArray::I32(a) => {
                downcast_vec::<T, i32>(owned(a)).map_err(|v| PackedArray::I32(Arc::new(v)))
            }
            PackedArray::I64(a) => {
                downcast_vec::<T, i64>(owned(a)).map_err(|v| PackedArray::I64(Arc::new(v)))
            }
            PackedArray::I128(a) => {
                downcast_vec::<T, i128>(owned(a)).map_err(|v| PackedArray::I128(Arc::new(v)))
            }
            PackedArray::U8(a) => {
                downcast_vec::<T, u8>(owned(a)).map_err(|v| PackedArray::U8(Arc::new(v)))
            }
            PackedArray::U16(a) => {
                downcast_vec::<T, u16>(owned(a)).map_err(|v| PackedArray::U16(Arc::new(v)))
            }
            PackedArray::U32(a) => {
                downcast_vec::<T, u32>(owned(a)).map_err(|v| PackedArray::U32(Arc::new(v)))
            }
            PackedArray::U64(a) => {
                downcast_vec::<T, u64>(owned(a)).map_err(|v| PackedArray::U64(Arc::new(v)))
            }
            PackedArray::U128(a) => {
                downcast_vec::<T, u128>(owned(a)).map_err(|v| PackedArray::U128(Arc::new(v)))
            }
            PackedArray::F32(a) => {
                downcast_vec::<T, f32>(owned(a)).map_err(|v| PackedArray::F32(Arc::new(v)))
            }
            PackedArray::F64(a) => {
                downcast_vec::<T, f64>(owned(a)).map_err(|v| PackedArray::F64(Arc::new(v)))
            }
            PackedArray::Bool(a) => {
                downcast_vec::<T, bool>(owned(a)).map_err(|v| PackedArray::Bool(Arc::new(v)))
            }
            PackedArray::Byte(a) => {
                downcast_vec::<T, u8>(owned(a)).map_err(|v| PackedArray::Byte(Arc::new(v)))
            }
            PackedArray::Char(a) => {
                downcast_vec::<T, char>(owned(a)).map_err(|v| PackedArray::Char(Arc::new(v)))
            }
        }
    }
}

impl From<PackedArray> for Value {
    fn from(value: PackedArray) -> Self {
        Value::Packed(value)
    }
}

/// Direct bridge from an already-packed array to `Value`, for `send_one_as`/callers
/// that already hold an `Arc<Vec<T>>` (e.g. forwarding one received via
/// `GetData<Arc<Vec<T>>>` without ever unpacking it) — composes through `PackedArray`'s
/// own per-type `From<Arc<Vec<T>>>` impls, so it's only available for `T`s that are
/// actually packable.
impl<T> From<Arc<Vec<T>>> for Value
where
    PackedArray: From<Arc<Vec<T>>>,
{
    fn from(value: Arc<Vec<T>>) -> Self {
        Value::Packed(PackedArray::from(value))
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

// `From<Vec<T>>`/`From<Arc<Vec<T>>> for PackedArray` and the matching
// `GetData<Arc<Vec<T>>> for Value` fast extractor follow the exact same shape for every
// scalar type, so they're generated rather than hand-duplicated — see ticket #120's
// precedent for `TransmissionValue`. `U8` is the one type kept hand-written below, for
// the same reason it is in `TransmissionValue`: `Byte` and `U8` are both `u8`-backed but
// semantically distinct (mel `byte` vs a generic 8-bit integer), so `From<Vec<u8>>` must
// stay unambiguous (always `U8`) while extraction accepts either.
macro_rules! packed_array_scalar_type {
    ($variant:ident, $ty:ty) => {
        impl From<Vec<$ty>> for PackedArray {
            fn from(value: Vec<$ty>) -> Self {
                PackedArray::$variant(Arc::new(value))
            }
        }

        impl From<Arc<Vec<$ty>>> for PackedArray {
            fn from(value: Arc<Vec<$ty>>) -> Self {
                PackedArray::$variant(value)
            }
        }

        impl GetData<Arc<Vec<$ty>>> for Value {
            fn try_data(self) -> Result<Arc<Vec<$ty>>, ()> {
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
        PackedArray::U8(Arc::new(value))
    }
}

impl From<Arc<Vec<u8>>> for PackedArray {
    fn from(value: Arc<Vec<u8>>) -> Self {
        PackedArray::U8(value)
    }
}

impl GetData<Arc<Vec<u8>>> for Value {
    fn try_data(self) -> Result<Arc<Vec<u8>>, ()> {
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
        let arr = PackedArray::U8(Arc::new(vec![0u8; 4096]));
        assert_eq!(arr.estimated_size(), 4096);

        let arr = PackedArray::F64(Arc::new(vec![0f64; 10]));
        assert_eq!(arr.estimated_size(), 10 * std::mem::size_of::<f64>());
    }

    #[test]
    fn value_packed_reports_the_same_datatype_as_value_vec() {
        let packed = Value::Packed(PackedArray::Byte(Arc::new(vec![1u8, 2, 3])));
        let boxed = Value::Vec(vec![Value::Byte(1), Value::Byte(2), Value::Byte(3)]);
        assert_eq!(packed.datatype(), boxed.datatype());
    }

    #[test]
    fn value_packed_estimated_size_has_no_per_element_value_overhead() {
        let base = std::mem::size_of::<Value>();
        let value = Value::Packed(PackedArray::Byte(Arc::new(vec![0u8; 4096])));
        // Contrast with `Value::Vec(Vec<Value::Byte>)`, which costs `base + n * base`
        // for the same content (see `estimated_size_tests::vec_sums_enum_footprint_of_every_element`).
        assert_eq!(value.estimated_size(), base + 4096);
    }

    #[test]
    fn arc_u8_extraction_accepts_both_u8_and_byte_variants() {
        let as_u8 = Value::Packed(PackedArray::U8(Arc::new(vec![1u8, 2, 3])));
        let as_byte = Value::Packed(PackedArray::Byte(Arc::new(vec![4u8, 5, 6])));

        let from_u8: Arc<Vec<u8>> = as_u8.try_data().unwrap();
        let from_byte: Arc<Vec<u8>> = as_byte.try_data().unwrap();

        assert_eq!(&*from_u8, &vec![1, 2, 3]);
        assert_eq!(&*from_byte, &vec![4, 5, 6]);
    }

    #[test]
    fn from_vec_u8_always_produces_u8_not_byte() {
        let packed: PackedArray = vec![1u8, 2, 3].into();
        assert!(matches!(packed, PackedArray::U8(_)));
    }

    #[test]
    fn mismatched_type_extraction_fails() {
        let value = Value::Packed(PackedArray::I64(Arc::new(vec![1i64])));
        let result: Result<Arc<Vec<u8>>, ()> = value.try_data();
        assert!(result.is_err());
    }

    #[test]
    fn into_values_roundtrips_element_by_element() {
        let packed = PackedArray::Byte(Arc::new(vec![1u8, 2, 3]));
        assert_eq!(
            packed.into_values(),
            vec![Value::Byte(1), Value::Byte(2), Value::Byte(3)]
        );
    }

    #[test]
    fn try_from_vec_packs_a_recognized_primitive_type() {
        let packed = PackedArray::try_from_vec(vec![1u8, 2, 3]).unwrap();
        assert!(matches!(packed, PackedArray::U8(_)));
    }

    #[test]
    fn try_from_vec_hands_back_the_vec_unchanged_for_a_non_packable_type() {
        let original = vec!["a".to_string(), "b".to_string()];
        let result = PackedArray::try_from_vec(original.clone());
        assert_eq!(result, Err(original));
    }

    #[test]
    fn try_into_vec_extracts_without_creating_any_value() {
        let packed = PackedArray::I64(Arc::new(vec![1i64, 2, 3]));
        let extracted: Vec<i64> = packed.try_into_vec().unwrap();
        assert_eq!(extracted, vec![1, 2, 3]);
    }

    #[test]
    fn try_into_vec_hands_back_the_packed_array_unchanged_on_mismatch() {
        let packed = PackedArray::I64(Arc::new(vec![1i64, 2, 3]));
        let result = packed.clone().try_into_vec::<u8>();
        assert_eq!(result, Err(packed));
    }

    #[test]
    fn arc_vec_converts_directly_into_value_packed() {
        let arc = Arc::new(vec![1u8, 2, 3]);
        let value: Value = arc.into();
        assert!(matches!(value, Value::Packed(PackedArray::U8(_))));
    }

    #[test]
    fn try_into_vec_does_not_copy_when_exclusively_owned() {
        let arc = Arc::new(vec![1u8, 2, 3]);
        let data_ptr = arc.as_ptr();
        let packed = PackedArray::U8(arc);
        let extracted: Vec<u8> = packed.try_into_vec().unwrap();
        assert_eq!(extracted.as_ptr(), data_ptr);
    }
}
