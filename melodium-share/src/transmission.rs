use crate::RawValue;
use melodium_common::descriptor::Collection;
use melodium_common::executive::TransmissionValue as CommonTransmissionValue;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Wire-serializable mirror of `melodium_common::executive::TransmissionValue`: a whole
/// *batch* of same-shaped values (one connection's worth of stream ticks), not one value
/// at a time like `RawValue`.
///
/// This exists specifically to close a gap `RawValue` alone leaves open: a `Stream<T>`
/// batch arrives in-process already packed as one `TransmissionValue::T(VecDeque<T>)` —
/// but converting it to a `Vec<RawValue>` (one `RawValue` per tick, `RawValue` having no
/// batching concept of its own) throws that packing away right before it hits the wire,
/// so every scalar tick ends up individually CBOR-tagged again. Sending this type
/// directly instead keeps a whole batch as one CBOR byte string / array, mirroring the
/// in-process representation all the way to the wire.
///
/// Deliberately a distinct type from `RawValue`, not an extension of it: `RawValue` is
/// also the value type exposed directly over WASM (`tsify`, see `value.rs`) and used for
/// saved program designs — a public, stable contract that batching is an internal wire
/// optimization with no business leaking into (see ticket #116's `RawValue::Bytes`/
/// `Packed*` revert, for exactly this reason). This type is used only by
/// `melodium-distribution`'s `InputData`/`OutputData` messages, never exposed over WASM.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransmissionValue {
    Void(Vec<()>),

    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    I128(Vec<i128>),

    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),
    U128(Vec<u128>),

    F32(Vec<f32>),
    F64(Vec<f64>),

    Bool(Vec<bool>),
    /// A batch of `Stream<byte>` ticks — encodes as a native CBOR byte string (the same
    /// reasoning as the old `RawValue::Bytes`, just correctly scoped to the wire-only
    /// batch type this time): one length-prefixed buffer for the whole batch, not one
    /// CBOR-tagged item per byte.
    #[serde(with = "serde_bytes")]
    Byte(Vec<u8>),
    Char(Vec<char>),
    String(Vec<String>),

    // The `Packed*` variants are the batch-of-*arrays* counterpart: each entry is one
    // whole `Stream<Vec<T>>` tick (or a lone `Vec<T>` value), not one scalar — mirroring
    // `melodium_common::TransmissionValue`'s own `Packed*` shape, which this type is a
    // direct wire counterpart of. `PackedByte`'s inner arrays get the same native-byte-
    // string treatment as `Byte` above, per-array.
    PackedI8(Vec<Vec<i8>>),
    PackedI16(Vec<Vec<i16>>),
    PackedI32(Vec<Vec<i32>>),
    PackedI64(Vec<Vec<i64>>),
    PackedI128(Vec<Vec<i128>>),

    PackedU8(Vec<Vec<u8>>),
    PackedU16(Vec<Vec<u16>>),
    PackedU32(Vec<Vec<u32>>),
    PackedU64(Vec<Vec<u64>>),
    PackedU128(Vec<Vec<u128>>),

    PackedF32(Vec<Vec<f32>>),
    PackedF64(Vec<Vec<f64>>),

    PackedBool(Vec<Vec<bool>>),
    PackedByte(Vec<serde_bytes::ByteBuf>),
    PackedChar(Vec<Vec<char>>),

    /// Fallback for anything without a dedicated batched representation above (mixed/
    /// custom `Data` batches, `Option<T>` batches, ...) — one `RawValue` per tick, same
    /// role as `melodium_common::TransmissionValue::Other`.
    Other(Vec<RawValue>),
}

impl TransmissionValue {
    /// Number of ticks in this batch (arrays count as one tick each, same as the
    /// in-process `TransmissionValue::len()`).
    pub fn len(&self) -> usize {
        match self {
            TransmissionValue::Void(v) => v.len(),
            TransmissionValue::I8(v) => v.len(),
            TransmissionValue::I16(v) => v.len(),
            TransmissionValue::I32(v) => v.len(),
            TransmissionValue::I64(v) => v.len(),
            TransmissionValue::I128(v) => v.len(),
            TransmissionValue::U8(v) => v.len(),
            TransmissionValue::U16(v) => v.len(),
            TransmissionValue::U32(v) => v.len(),
            TransmissionValue::U64(v) => v.len(),
            TransmissionValue::U128(v) => v.len(),
            TransmissionValue::F32(v) => v.len(),
            TransmissionValue::F64(v) => v.len(),
            TransmissionValue::Bool(v) => v.len(),
            TransmissionValue::Byte(v) => v.len(),
            TransmissionValue::Char(v) => v.len(),
            TransmissionValue::String(v) => v.len(),
            TransmissionValue::PackedI8(v) => v.len(),
            TransmissionValue::PackedI16(v) => v.len(),
            TransmissionValue::PackedI32(v) => v.len(),
            TransmissionValue::PackedI64(v) => v.len(),
            TransmissionValue::PackedI128(v) => v.len(),
            TransmissionValue::PackedU8(v) => v.len(),
            TransmissionValue::PackedU16(v) => v.len(),
            TransmissionValue::PackedU32(v) => v.len(),
            TransmissionValue::PackedU64(v) => v.len(),
            TransmissionValue::PackedU128(v) => v.len(),
            TransmissionValue::PackedF32(v) => v.len(),
            TransmissionValue::PackedF64(v) => v.len(),
            TransmissionValue::PackedBool(v) => v.len(),
            TransmissionValue::PackedByte(v) => v.len(),
            TransmissionValue::PackedChar(v) => v.len(),
            TransmissionValue::Other(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Rough on-wire footprint of the whole batch, in bytes — used to size-cap a single
    /// `InputData`/`OutputData` message (see `melodium-distribution`'s framing), so it
    /// favors being cheap over exact, same spirit as `RawValue::estimated_size`.
    pub fn estimated_size(&self) -> usize {
        match self {
            TransmissionValue::Void(v) => v.len() * std::mem::size_of::<()>(),
            TransmissionValue::I8(v) => v.len() * std::mem::size_of::<i8>(),
            TransmissionValue::I16(v) => v.len() * std::mem::size_of::<i16>(),
            TransmissionValue::I32(v) => v.len() * std::mem::size_of::<i32>(),
            TransmissionValue::I64(v) => v.len() * std::mem::size_of::<i64>(),
            TransmissionValue::I128(v) => v.len() * std::mem::size_of::<i128>(),
            TransmissionValue::U8(v) => v.len() * std::mem::size_of::<u8>(),
            TransmissionValue::U16(v) => v.len() * std::mem::size_of::<u16>(),
            TransmissionValue::U32(v) => v.len() * std::mem::size_of::<u32>(),
            TransmissionValue::U64(v) => v.len() * std::mem::size_of::<u64>(),
            TransmissionValue::U128(v) => v.len() * std::mem::size_of::<u128>(),
            TransmissionValue::F32(v) => v.len() * std::mem::size_of::<f32>(),
            TransmissionValue::F64(v) => v.len() * std::mem::size_of::<f64>(),
            TransmissionValue::Bool(v) => v.len() * std::mem::size_of::<bool>(),
            TransmissionValue::Byte(v) => v.len(),
            TransmissionValue::Char(v) => v.len() * std::mem::size_of::<char>(),
            TransmissionValue::String(v) => v.iter().map(String::len).sum(),
            TransmissionValue::PackedI8(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<i8>()).sum()
            }
            TransmissionValue::PackedI16(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<i16>()).sum()
            }
            TransmissionValue::PackedI32(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<i32>()).sum()
            }
            TransmissionValue::PackedI64(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<i64>()).sum()
            }
            TransmissionValue::PackedI128(v) => v
                .iter()
                .map(|a| a.len() * std::mem::size_of::<i128>())
                .sum(),
            TransmissionValue::PackedU8(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<u8>()).sum()
            }
            TransmissionValue::PackedU16(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<u16>()).sum()
            }
            TransmissionValue::PackedU32(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<u32>()).sum()
            }
            TransmissionValue::PackedU64(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<u64>()).sum()
            }
            TransmissionValue::PackedU128(v) => v
                .iter()
                .map(|a| a.len() * std::mem::size_of::<u128>())
                .sum(),
            TransmissionValue::PackedF32(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<f32>()).sum()
            }
            TransmissionValue::PackedF64(v) => {
                v.iter().map(|a| a.len() * std::mem::size_of::<f64>()).sum()
            }
            TransmissionValue::PackedBool(v) => v
                .iter()
                .map(|a| a.len() * std::mem::size_of::<bool>())
                .sum(),
            TransmissionValue::PackedByte(v) => v.iter().map(|a| a.len()).sum(),
            TransmissionValue::PackedChar(v) => v
                .iter()
                .map(|a| a.len() * std::mem::size_of::<char>())
                .sum(),
            TransmissionValue::Other(v) => v.iter().map(RawValue::estimated_size).sum(),
        }
    }

    /// Splits into size-capped chunks, each becoming its own `InputData`/`OutputData`
    /// message — the batch-type equivalent of the old `chunk_raw_values`, but operating
    /// on one homogeneous buffer instead of a heterogeneous list, so it's just `.chunks()`
    /// by estimated running size rather than needing to reason about mixed element sizes.
    /// Every chunk holds at least one tick, even if that one tick alone exceeds
    /// `max_bytes` (this bounds batches, not individual ticks). An empty batch yields no
    /// chunks.
    pub fn chunked(self, max_bytes: usize) -> Vec<TransmissionValue> {
        fn chunk_by_size<T>(
            items: Vec<T>,
            max_bytes: usize,
            size_of: impl Fn(&T) -> usize,
        ) -> Vec<Vec<T>> {
            let mut chunks = Vec::new();
            let mut current = Vec::new();
            let mut current_bytes = 0usize;

            for item in items {
                let item_bytes = size_of(&item);
                if !current.is_empty() && current_bytes + item_bytes > max_bytes {
                    chunks.push(std::mem::take(&mut current));
                    current_bytes = 0;
                }
                current_bytes += item_bytes;
                current.push(item);
            }

            if !current.is_empty() {
                chunks.push(current);
            }

            chunks
        }

        macro_rules! chunk_fixed {
            ($v:expr, $variant:ident, $ty:ty) => {
                chunk_by_size($v, max_bytes, |_| std::mem::size_of::<$ty>())
                    .into_iter()
                    .map(TransmissionValue::$variant)
                    .collect()
            };
        }
        macro_rules! chunk_packed {
            ($v:expr, $variant:ident, $ty:ty) => {
                chunk_by_size($v, max_bytes, |a: &Vec<$ty>| {
                    a.len() * std::mem::size_of::<$ty>()
                })
                .into_iter()
                .map(TransmissionValue::$variant)
                .collect()
            };
        }

        match self {
            TransmissionValue::Void(v) => chunk_fixed!(v, Void, ()),
            TransmissionValue::I8(v) => chunk_fixed!(v, I8, i8),
            TransmissionValue::I16(v) => chunk_fixed!(v, I16, i16),
            TransmissionValue::I32(v) => chunk_fixed!(v, I32, i32),
            TransmissionValue::I64(v) => chunk_fixed!(v, I64, i64),
            TransmissionValue::I128(v) => chunk_fixed!(v, I128, i128),
            TransmissionValue::U8(v) => chunk_fixed!(v, U8, u8),
            TransmissionValue::U16(v) => chunk_fixed!(v, U16, u16),
            TransmissionValue::U32(v) => chunk_fixed!(v, U32, u32),
            TransmissionValue::U64(v) => chunk_fixed!(v, U64, u64),
            TransmissionValue::U128(v) => chunk_fixed!(v, U128, u128),
            TransmissionValue::F32(v) => chunk_fixed!(v, F32, f32),
            TransmissionValue::F64(v) => chunk_fixed!(v, F64, f64),
            TransmissionValue::Bool(v) => chunk_fixed!(v, Bool, bool),
            TransmissionValue::Byte(v) => chunk_by_size(v, max_bytes, |_| 1)
                .into_iter()
                .map(TransmissionValue::Byte)
                .collect(),
            TransmissionValue::Char(v) => chunk_fixed!(v, Char, char),
            TransmissionValue::String(v) => chunk_by_size(v, max_bytes, String::len)
                .into_iter()
                .map(TransmissionValue::String)
                .collect(),
            TransmissionValue::PackedI8(v) => chunk_packed!(v, PackedI8, i8),
            TransmissionValue::PackedI16(v) => chunk_packed!(v, PackedI16, i16),
            TransmissionValue::PackedI32(v) => chunk_packed!(v, PackedI32, i32),
            TransmissionValue::PackedI64(v) => chunk_packed!(v, PackedI64, i64),
            TransmissionValue::PackedI128(v) => chunk_packed!(v, PackedI128, i128),
            TransmissionValue::PackedU8(v) => chunk_packed!(v, PackedU8, u8),
            TransmissionValue::PackedU16(v) => chunk_packed!(v, PackedU16, u16),
            TransmissionValue::PackedU32(v) => chunk_packed!(v, PackedU32, u32),
            TransmissionValue::PackedU64(v) => chunk_packed!(v, PackedU64, u64),
            TransmissionValue::PackedU128(v) => chunk_packed!(v, PackedU128, u128),
            TransmissionValue::PackedF32(v) => chunk_packed!(v, PackedF32, f32),
            TransmissionValue::PackedF64(v) => chunk_packed!(v, PackedF64, f64),
            TransmissionValue::PackedBool(v) => chunk_packed!(v, PackedBool, bool),
            TransmissionValue::PackedByte(v) => chunk_by_size(v, max_bytes, |a| a.len())
                .into_iter()
                .map(TransmissionValue::PackedByte)
                .collect(),
            TransmissionValue::PackedChar(v) => chunk_packed!(v, PackedChar, char),
            TransmissionValue::Other(v) => chunk_by_size(v, max_bytes, RawValue::estimated_size)
                .into_iter()
                .map(TransmissionValue::Other)
                .collect(),
        }
    }
}

impl From<CommonTransmissionValue> for TransmissionValue {
    fn from(value: CommonTransmissionValue) -> Self {
        match value {
            CommonTransmissionValue::Void(v) => TransmissionValue::Void(v.into()),
            CommonTransmissionValue::I8(v) => TransmissionValue::I8(v.into()),
            CommonTransmissionValue::I16(v) => TransmissionValue::I16(v.into()),
            CommonTransmissionValue::I32(v) => TransmissionValue::I32(v.into()),
            CommonTransmissionValue::I64(v) => TransmissionValue::I64(v.into()),
            CommonTransmissionValue::I128(v) => TransmissionValue::I128(v.into()),
            CommonTransmissionValue::U8(v) => TransmissionValue::U8(v.into()),
            CommonTransmissionValue::U16(v) => TransmissionValue::U16(v.into()),
            CommonTransmissionValue::U32(v) => TransmissionValue::U32(v.into()),
            CommonTransmissionValue::U64(v) => TransmissionValue::U64(v.into()),
            CommonTransmissionValue::U128(v) => TransmissionValue::U128(v.into()),
            CommonTransmissionValue::F32(v) => TransmissionValue::F32(v.into()),
            CommonTransmissionValue::F64(v) => TransmissionValue::F64(v.into()),
            CommonTransmissionValue::Bool(v) => TransmissionValue::Bool(v.into()),
            CommonTransmissionValue::Byte(v) => TransmissionValue::Byte(v.into()),
            CommonTransmissionValue::Char(v) => TransmissionValue::Char(v.into()),
            CommonTransmissionValue::String(v) => TransmissionValue::String(v.into()),
            CommonTransmissionValue::PackedI8(v) => {
                TransmissionValue::PackedI8(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedI16(v) => {
                TransmissionValue::PackedI16(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedI32(v) => {
                TransmissionValue::PackedI32(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedI64(v) => {
                TransmissionValue::PackedI64(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedI128(v) => {
                TransmissionValue::PackedI128(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedU8(v) => {
                TransmissionValue::PackedU8(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedU16(v) => {
                TransmissionValue::PackedU16(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedU32(v) => {
                TransmissionValue::PackedU32(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedU64(v) => {
                TransmissionValue::PackedU64(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedU128(v) => {
                TransmissionValue::PackedU128(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedF32(v) => {
                TransmissionValue::PackedF32(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedF64(v) => {
                TransmissionValue::PackedF64(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedBool(v) => {
                TransmissionValue::PackedBool(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::PackedByte(v) => TransmissionValue::PackedByte(
                v.into_iter()
                    .map(|a| serde_bytes::ByteBuf::from((*a).clone()))
                    .collect(),
            ),
            CommonTransmissionValue::PackedChar(v) => {
                TransmissionValue::PackedChar(v.into_iter().map(|a| (*a).clone()).collect())
            }
            CommonTransmissionValue::Other(v) => {
                TransmissionValue::Other(v.into_iter().map(|value| value.into()).collect())
            }
        }
    }
}

impl TransmissionValue {
    /// Converts back to the in-process batch type. Takes a `&Collection` because the one
    /// variant that can hold custom `Data` values (`Other`, via `RawValue::Data`) needs it
    /// to find the right deserializer — exactly like `RawValue::to_value` does for a
    /// single value. Every other variant is plain primitives and never fails; `Other`
    /// fails (returns `None`) if any element can't be resolved against `collection`,
    /// mirroring `RawValue::to_value`'s own failure shape rather than silently dropping
    /// unresolvable elements.
    pub fn to_transmission_value(self, collection: &Collection) -> Option<CommonTransmissionValue> {
        Some(match self {
            TransmissionValue::Void(v) => CommonTransmissionValue::Void(v.into()),
            TransmissionValue::I8(v) => CommonTransmissionValue::I8(v.into()),
            TransmissionValue::I16(v) => CommonTransmissionValue::I16(v.into()),
            TransmissionValue::I32(v) => CommonTransmissionValue::I32(v.into()),
            TransmissionValue::I64(v) => CommonTransmissionValue::I64(v.into()),
            TransmissionValue::I128(v) => CommonTransmissionValue::I128(v.into()),
            TransmissionValue::U8(v) => CommonTransmissionValue::U8(v.into()),
            TransmissionValue::U16(v) => CommonTransmissionValue::U16(v.into()),
            TransmissionValue::U32(v) => CommonTransmissionValue::U32(v.into()),
            TransmissionValue::U64(v) => CommonTransmissionValue::U64(v.into()),
            TransmissionValue::U128(v) => CommonTransmissionValue::U128(v.into()),
            TransmissionValue::F32(v) => CommonTransmissionValue::F32(v.into()),
            TransmissionValue::F64(v) => CommonTransmissionValue::F64(v.into()),
            TransmissionValue::Bool(v) => CommonTransmissionValue::Bool(v.into()),
            TransmissionValue::Byte(v) => CommonTransmissionValue::Byte(v.into()),
            TransmissionValue::Char(v) => CommonTransmissionValue::Char(v.into()),
            TransmissionValue::String(v) => CommonTransmissionValue::String(v.into()),
            TransmissionValue::PackedI8(v) => {
                CommonTransmissionValue::PackedI8(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedI16(v) => {
                CommonTransmissionValue::PackedI16(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedI32(v) => {
                CommonTransmissionValue::PackedI32(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedI64(v) => {
                CommonTransmissionValue::PackedI64(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedI128(v) => {
                CommonTransmissionValue::PackedI128(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedU8(v) => {
                CommonTransmissionValue::PackedU8(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedU16(v) => {
                CommonTransmissionValue::PackedU16(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedU32(v) => {
                CommonTransmissionValue::PackedU32(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedU64(v) => {
                CommonTransmissionValue::PackedU64(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedU128(v) => {
                CommonTransmissionValue::PackedU128(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedF32(v) => {
                CommonTransmissionValue::PackedF32(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedF64(v) => {
                CommonTransmissionValue::PackedF64(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedBool(v) => {
                CommonTransmissionValue::PackedBool(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::PackedByte(v) => CommonTransmissionValue::PackedByte(
                v.into_iter().map(|a| Arc::new(a.into_vec())).collect(),
            ),
            TransmissionValue::PackedChar(v) => {
                CommonTransmissionValue::PackedChar(v.into_iter().map(Arc::new).collect())
            }
            TransmissionValue::Other(v) => {
                let mut values = std::collections::VecDeque::with_capacity(v.len());
                for value in v {
                    values.push_back(value.to_value(collection)?);
                }
                CommonTransmissionValue::Other(values)
            }
        })
    }
}

#[cfg(test)]
mod wire_tests {
    use super::*;
    use melodium_common::executive::{PackedArray, Value as CommonValue};
    use std::collections::VecDeque;

    // The entire point of this type: a batch of scalar `Stream<byte>` ticks must encode
    // as one native CBOR byte string for the whole batch, not one CBOR-tagged item per
    // byte - this is exactly the gap `RawValue` (no batching concept) leaves open at
    // `melodium-distribution/src/listen.rs`, proven there via `Into::<VecDeque<Value>>`
    // exploding a packed `TransmissionValue::Byte` batch back into individual ticks.
    #[test]
    fn byte_batch_encodes_as_a_native_cbor_byte_string_not_a_tagged_array_per_tick() {
        let payload = vec![0u8; 1000];
        let batch = TransmissionValue::Byte(payload.clone());
        let encoded =
            cbor4ii::serde::to_vec(Vec::new(), &batch).expect("serialization must succeed");

        assert!(
            encoded.len() < payload.len() + 32,
            "expected near-payload-sized encoding for a native byte string batch, got {} bytes for a {}-byte payload",
            encoded.len(),
            payload.len()
        );

        let decoded: TransmissionValue =
            cbor4ii::serde::from_slice(&encoded).expect("must round-trip");
        assert_eq!(decoded, batch);
    }

    // Same property for `PackedByte`: each array in the batch is its own native byte
    // string, not a CBOR array of tagged numbers.
    #[test]
    fn packed_byte_batch_encodes_each_array_as_a_native_cbor_byte_string() {
        let batch = TransmissionValue::PackedByte(vec![
            serde_bytes::ByteBuf::from(vec![0u8; 500]),
            serde_bytes::ByteBuf::from(vec![1u8; 500]),
        ]);
        let encoded =
            cbor4ii::serde::to_vec(Vec::new(), &batch).expect("serialization must succeed");

        assert!(
            encoded.len() < 1000 + 64,
            "expected near-payload-sized encoding for two native byte string arrays, got {} bytes",
            encoded.len()
        );

        let decoded: TransmissionValue =
            cbor4ii::serde::from_slice(&encoded).expect("must round-trip");
        assert_eq!(decoded, batch);
    }

    #[test]
    fn scalar_batch_roundtrips_through_cbor() {
        let batch = TransmissionValue::I64(vec![1, 2, 3]);
        let encoded =
            cbor4ii::serde::to_vec(Vec::new(), &batch).expect("serialization must succeed");
        let decoded: TransmissionValue =
            cbor4ii::serde::from_slice(&encoded).expect("must round-trip");
        assert_eq!(decoded, batch);
    }

    // The actual bug this whole type exists to fix: a `TransmissionValue::Byte` batch
    // (already packed in-process) must convert directly into the wire type as ONE
    // `TransmissionValue::Byte`, not explode per-tick the way going through
    // `Into::<VecDeque<Value>>` first would.
    #[test]
    fn common_byte_batch_converts_to_one_wire_byte_batch_not_per_tick_values() {
        let common = CommonTransmissionValue::Byte(VecDeque::from(vec![1u8, 2, 3]));
        let wire: TransmissionValue = common.into();
        assert_eq!(wire, TransmissionValue::Byte(vec![1, 2, 3]));
    }

    #[test]
    fn common_packed_batch_converts_to_wire_packed_batch() {
        let common = CommonTransmissionValue::PackedI64(VecDeque::from(vec![
            Arc::new(vec![1i64, 2, 3]),
            Arc::new(vec![4i64, 5]),
        ]));
        let wire: TransmissionValue = common.into();
        assert_eq!(
            wire,
            TransmissionValue::PackedI64(vec![vec![1, 2, 3], vec![4, 5]])
        );
    }

    #[test]
    fn wire_batch_converts_back_to_common_batch() {
        let collection = Collection::new();
        let wire = TransmissionValue::U64(vec![1, 2, 3]);
        let common = wire.to_transmission_value(&collection).unwrap();
        assert_eq!(
            common,
            CommonTransmissionValue::U64(VecDeque::from(vec![1, 2, 3]))
        );
    }

    #[test]
    fn other_batch_roundtrips_through_collection_aware_conversion() {
        let collection = Collection::new();
        let common = CommonTransmissionValue::Other(VecDeque::from(vec![
            CommonValue::String("a".to_string()),
            CommonValue::Bool(true),
        ]));
        let wire: TransmissionValue = common.clone().into();
        let back = wire.to_transmission_value(&collection).unwrap();
        assert_eq!(back, common);
    }

    #[test]
    fn chunked_splits_a_large_batch_and_preserves_every_tick() {
        let batch = TransmissionValue::U64(vec![1; 100]);
        let chunks = batch.chunked(std::mem::size_of::<u64>() * 10);
        assert!(chunks.len() > 1, "expected the batch to actually split");
        let total: usize = chunks.iter().map(TransmissionValue::len).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn chunked_keeps_a_small_batch_in_one_chunk() {
        let batch = TransmissionValue::U64(vec![1, 2, 3]);
        let chunks = batch.chunked(1024 * 1024);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 3);
    }

    #[test]
    fn chunked_never_produces_an_empty_chunk_even_for_an_oversized_single_tick() {
        let batch = TransmissionValue::String(vec!["x".repeat(10_000)]);
        let chunks = batch.chunked(100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 1);
    }

    #[test]
    fn value_packed_converts_through_common_transmission_value_correctly() {
        // Sanity check tying this together with `Value::Packed` end-to-end: a single
        // packed value, wrapped into a one-tick `TransmissionValue`, survives the
        // common -> wire -> common round trip intact.
        let value = CommonValue::Packed(PackedArray::Byte(Arc::new(vec![1u8, 2, 3])));
        let common = CommonTransmissionValue::new(value.clone());
        let wire: TransmissionValue = common.into();
        let collection = Collection::new();
        let back = wire.to_transmission_value(&collection).unwrap();
        assert_eq!(
            back,
            CommonTransmissionValue::PackedByte(VecDeque::from(vec![Arc::new(vec![1u8, 2, 3])]))
        );
        let _ = value;
    }
}
