use crate::{DescribedType, Identifier, SharingError, SharingResult};
use cbor4ii::core::utils::SliceReader;
use melodium_common::{
    descriptor::{Collection, Entry as CommonEntry, Identifier as CommonIdentifier},
    executive::{PackedArray as CommonPackedArray, Value as CommonValue},
};
use melodium_engine::{design::Value as DesignedValue, LogicError};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Value {
    Raw(RawValue),
    Array(Vec<Value>),
    Variable(String),
    Context(Identifier, String),
    Function(Identifier, BTreeMap<String, DescribedType>, Vec<Value>),
}

impl Value {
    pub fn to_value(
        &self,
        collection: &Collection,
        scope: &CommonIdentifier,
    ) -> SharingResult<DesignedValue> {
        match self {
            Value::Raw(val) => {
                if let Some(value) = val.to_value(collection) {
                    SharingResult::new_success(DesignedValue::Raw(value))
                } else {
                    SharingResult::new_failure(SharingError::data_serialization_error(8))
                }
            }
            Value::Array(arr) => {
                let mut result = SharingResult::new_success(());
                let mut vec = Vec::with_capacity(arr.len());
                for val in arr {
                    if let Some(val) = result.merge_degrade_failure(val.to_value(collection, scope))
                    {
                        vec.push(val);
                    }
                }
                result.and_then(|_| SharingResult::new_success(DesignedValue::Array(vec)))
            }
            Value::Variable(var) => {
                SharingResult::new_success(DesignedValue::Variable(var.clone()))
            }
            Value::Context(context, name) => {
                let context: CommonIdentifier = if let Ok(identifier) = context.try_into() {
                    identifier
                } else {
                    return SharingResult::new_failure(SharingError::invalid_identifier(
                        9,
                        context.clone(),
                    ));
                };
                if let Some(CommonEntry::Context(context)) = collection.get(&(&context).into()) {
                    SharingResult::new_success(DesignedValue::Context(
                        Arc::clone(context),
                        name.clone(),
                    ))
                } else {
                    SharingResult::new_failure(
                        LogicError::unexisting_context(232, scope.clone(), context.into(), None)
                            .into(),
                    )
                }
            }
            Value::Function(function, generics, parameters) => {
                let function: CommonIdentifier = if let Ok(identifier) = function.try_into() {
                    identifier
                } else {
                    return SharingResult::new_failure(SharingError::invalid_identifier(
                        10,
                        function.clone(),
                    ));
                };
                if let Some(CommonEntry::Function(function)) = collection.get(&(&function).into()) {
                    let mut result = SharingResult::new_success(());

                    let mut map_generics = HashMap::with_capacity(generics.len());
                    for (name, gen) in generics {
                        if let Some(gen) =
                            result.merge_degrade_failure(gen.to_described_type(collection, scope))
                        {
                            map_generics.insert(name.clone(), gen);
                        }
                    }

                    let mut vec_params = Vec::with_capacity(parameters.len());
                    for param in parameters {
                        if let Some(val) =
                            result.merge_degrade_failure(param.to_value(collection, scope))
                        {
                            vec_params.push(val);
                        }
                    }

                    result.and_then(|_| {
                        SharingResult::new_success(DesignedValue::Function(
                            Arc::clone(function),
                            map_generics,
                            vec_params,
                        ))
                    })
                } else {
                    SharingResult::new_failure(
                        LogicError::unexisting_function(233, scope.clone(), function.into(), None)
                            .into(),
                    )
                }
            }
        }
    }
}

impl From<&DesignedValue> for Value {
    fn from(value: &DesignedValue) -> Self {
        match value {
            DesignedValue::Raw(val) => Value::Raw(val.into()),
            DesignedValue::Array(arr) => Value::Array(arr.iter().map(|v| v.into()).collect()),
            DesignedValue::Variable(var) => Value::Variable(var.clone()),
            DesignedValue::Context(context, name) => {
                Value::Context(context.identifier().into(), name.clone())
            }
            DesignedValue::Function(function, generics, params) => Value::Function(
                function.identifier().into(),
                generics
                    .iter()
                    .map(|(name, dt)| (name.clone(), dt.into()))
                    .collect(),
                params.iter().map(|p| p.into()).collect(),
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum RawValue {
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

    Vec(Vec<RawValue>),
    Option(Option<Box<RawValue>>),

    // The `Packed*` variants below are the wire counterpart of `CommonValue::Packed`
    // (see melodium-common's `PackedArray`, ticket #116): one `RawValue` for a whole
    // homogeneous scalar array instead of one `RawValue` per element. `Bytes` is the
    // one case with its own native CBOR encoding (a byte string, via `serde_bytes`)
    // rather than a plain array — that's where the per-element CBOR tag overhead is
    // worst (up to 2x for a 1-byte payload), and CBOR already has a type built for it.
    // The others just save the `RawValue` enum overhead per element (no more per-array
    // CBOR tag/discriminant multiplied by array length) without a custom binary layout.
    #[serde(with = "serde_bytes")]
    Bytes(Vec<u8>),
    PackedI8(Vec<i8>),
    PackedI16(Vec<i16>),
    PackedI32(Vec<i32>),
    PackedI64(Vec<i64>),
    PackedI128(Vec<i128>),
    PackedU8(Vec<u8>),
    PackedU16(Vec<u16>),
    PackedU32(Vec<u32>),
    PackedU64(Vec<u64>),
    PackedU128(Vec<u128>),
    PackedF32(Vec<f32>),
    PackedF64(Vec<f64>),
    PackedBool(Vec<bool>),
    PackedChar(Vec<char>),

    Data(Identifier, Option<Vec<u8>>),
}

impl RawValue {
    pub fn to_value(&self, collection: &Collection) -> Option<CommonValue> {
        match self {
            RawValue::Data(identifier, value) => {
                if let Ok(identifier) =
                    <&Identifier as TryInto<CommonIdentifier>>::try_into(identifier)
                {
                    match (collection.get(&(&identifier).into()), value) {
                        (Some(CommonEntry::Data(data)), Some(value)) => {
                            let slice_reader = SliceReader::new(value.as_slice());

                            let mut deserializer_cbor =
                                cbor4ii::serde::Deserializer::new(slice_reader);
                            let mut erased_deserializer = Box::new(
                                <dyn erased_serde::Deserializer>::erase(&mut deserializer_cbor),
                            );

                            data.deserialize(&mut erased_deserializer).ok()
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            RawValue::Vec(v) => Some({
                let mut vec = Vec::with_capacity(v.len());
                for val in v {
                    vec.push(val.to_value(collection)?);
                }
                CommonValue::Vec(vec)
            }),
            RawValue::Option(option) => Some(match option {
                None => CommonValue::Option(None),
                Some(value) => CommonValue::Option(Some(Box::new(value.to_value(collection)?))),
            }),
            other => other.try_into().ok(),
        }
    }

    /// Rough on-wire footprint of this value, in bytes — used to keep a single
    /// `InputData`/`OutputData` message bounded (see `melodium-distribution`'s framing),
    /// so it favors being cheap to compute over being exact. Every `RawValue` occupies
    /// `size_of::<RawValue>()` inline regardless of variant, plus whatever it owns on the
    /// heap. `Data` is the one case this can size exactly rather than estimate: it already
    /// carries its own pre-serialized CBOR bytes.
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<RawValue>()
            + match self {
                RawValue::String(value) => value.len(),
                RawValue::Vec(values) => values.iter().map(RawValue::estimated_size).sum(),
                RawValue::Option(Some(value)) => value.estimated_size(),
                RawValue::Bytes(value) => value.len(),
                RawValue::PackedI8(value) => value.len() * std::mem::size_of::<i8>(),
                RawValue::PackedI16(value) => value.len() * std::mem::size_of::<i16>(),
                RawValue::PackedI32(value) => value.len() * std::mem::size_of::<i32>(),
                RawValue::PackedI64(value) => value.len() * std::mem::size_of::<i64>(),
                RawValue::PackedI128(value) => value.len() * std::mem::size_of::<i128>(),
                RawValue::PackedU8(value) => value.len() * std::mem::size_of::<u8>(),
                RawValue::PackedU16(value) => value.len() * std::mem::size_of::<u16>(),
                RawValue::PackedU32(value) => value.len() * std::mem::size_of::<u32>(),
                RawValue::PackedU64(value) => value.len() * std::mem::size_of::<u64>(),
                RawValue::PackedU128(value) => value.len() * std::mem::size_of::<u128>(),
                RawValue::PackedF32(value) => value.len() * std::mem::size_of::<f32>(),
                RawValue::PackedF64(value) => value.len() * std::mem::size_of::<f64>(),
                RawValue::PackedBool(value) => value.len() * std::mem::size_of::<bool>(),
                RawValue::PackedChar(value) => value.len() * std::mem::size_of::<char>(),
                RawValue::Data(_, value) => value.as_ref().map(Vec::len).unwrap_or(0),
                _ => 0,
            }
    }
}

impl From<CommonValue> for RawValue {
    fn from(value: CommonValue) -> Self {
        match value {
            CommonValue::Void(_) => RawValue::Void(()),
            CommonValue::I8(n) => RawValue::I8(n),
            CommonValue::I16(n) => RawValue::I16(n),
            CommonValue::I32(n) => RawValue::I32(n),
            CommonValue::I64(n) => RawValue::I64(n),
            CommonValue::I128(n) => RawValue::I128(n),
            CommonValue::U8(n) => RawValue::U8(n),
            CommonValue::U16(n) => RawValue::U16(n),
            CommonValue::U32(n) => RawValue::U32(n),
            CommonValue::U64(n) => RawValue::U64(n),
            CommonValue::U128(n) => RawValue::U128(n),
            CommonValue::F32(n) => RawValue::F32(n),
            CommonValue::F64(n) => RawValue::F64(n),
            CommonValue::Bool(b) => RawValue::Bool(b),
            CommonValue::Byte(b) => RawValue::Byte(b),
            CommonValue::Char(c) => RawValue::Char(c),
            CommonValue::String(s) => RawValue::String(s),
            CommonValue::Vec(v) => RawValue::Vec(v.into_iter().map(|v| v.into()).collect()),
            CommonValue::Option(v) => RawValue::Option(v.map(|v| Box::new((*v).into()))),
            CommonValue::Packed(arr) => match arr {
                CommonPackedArray::I8(a) => RawValue::PackedI8(a.to_vec()),
                CommonPackedArray::I16(a) => RawValue::PackedI16(a.to_vec()),
                CommonPackedArray::I32(a) => RawValue::PackedI32(a.to_vec()),
                CommonPackedArray::I64(a) => RawValue::PackedI64(a.to_vec()),
                CommonPackedArray::I128(a) => RawValue::PackedI128(a.to_vec()),
                CommonPackedArray::U8(a) => RawValue::PackedU8(a.to_vec()),
                CommonPackedArray::U16(a) => RawValue::PackedU16(a.to_vec()),
                CommonPackedArray::U32(a) => RawValue::PackedU32(a.to_vec()),
                CommonPackedArray::U64(a) => RawValue::PackedU64(a.to_vec()),
                CommonPackedArray::U128(a) => RawValue::PackedU128(a.to_vec()),
                CommonPackedArray::F32(a) => RawValue::PackedF32(a.to_vec()),
                CommonPackedArray::F64(a) => RawValue::PackedF64(a.to_vec()),
                CommonPackedArray::Bool(a) => RawValue::PackedBool(a.to_vec()),
                CommonPackedArray::Byte(a) => RawValue::Bytes(a.to_vec()),
                CommonPackedArray::Char(a) => RawValue::PackedChar(a.to_vec()),
            },
            CommonValue::Data(d) => {
                let data = cbor4ii::serde::to_vec(Vec::new(), &d).ok();
                RawValue::Data(d.descriptor().identifier().into(), data)
            }
        }
    }
}

impl From<&CommonValue> for RawValue {
    fn from(value: &CommonValue) -> Self {
        match value {
            CommonValue::Void(_) => RawValue::Void(()),
            CommonValue::I8(n) => RawValue::I8(*n),
            CommonValue::I16(n) => RawValue::I16(*n),
            CommonValue::I32(n) => RawValue::I32(*n),
            CommonValue::I64(n) => RawValue::I64(*n),
            CommonValue::I128(n) => RawValue::I128(*n),
            CommonValue::U8(n) => RawValue::U8(*n),
            CommonValue::U16(n) => RawValue::U16(*n),
            CommonValue::U32(n) => RawValue::U32(*n),
            CommonValue::U64(n) => RawValue::U64(*n),
            CommonValue::U128(n) => RawValue::U128(*n),
            CommonValue::F32(n) => RawValue::F32(*n),
            CommonValue::F64(n) => RawValue::F64(*n),
            CommonValue::Bool(b) => RawValue::Bool(*b),
            CommonValue::Byte(b) => RawValue::Byte(*b),
            CommonValue::Char(c) => RawValue::Char(*c),
            CommonValue::String(s) => RawValue::String(s.clone()),
            CommonValue::Vec(v) => RawValue::Vec(v.into_iter().map(|v| v.into()).collect()),
            CommonValue::Option(v) => {
                RawValue::Option(v.as_ref().map(|v| Box::new(v.as_ref().into())))
            }
            CommonValue::Packed(arr) => match arr {
                CommonPackedArray::I8(a) => RawValue::PackedI8(a.to_vec()),
                CommonPackedArray::I16(a) => RawValue::PackedI16(a.to_vec()),
                CommonPackedArray::I32(a) => RawValue::PackedI32(a.to_vec()),
                CommonPackedArray::I64(a) => RawValue::PackedI64(a.to_vec()),
                CommonPackedArray::I128(a) => RawValue::PackedI128(a.to_vec()),
                CommonPackedArray::U8(a) => RawValue::PackedU8(a.to_vec()),
                CommonPackedArray::U16(a) => RawValue::PackedU16(a.to_vec()),
                CommonPackedArray::U32(a) => RawValue::PackedU32(a.to_vec()),
                CommonPackedArray::U64(a) => RawValue::PackedU64(a.to_vec()),
                CommonPackedArray::U128(a) => RawValue::PackedU128(a.to_vec()),
                CommonPackedArray::F32(a) => RawValue::PackedF32(a.to_vec()),
                CommonPackedArray::F64(a) => RawValue::PackedF64(a.to_vec()),
                CommonPackedArray::Bool(a) => RawValue::PackedBool(a.to_vec()),
                CommonPackedArray::Byte(a) => RawValue::Bytes(a.to_vec()),
                CommonPackedArray::Char(a) => RawValue::PackedChar(a.to_vec()),
            },
            CommonValue::Data(d) => {
                let data = cbor4ii::serde::to_vec(Vec::new(), &d).ok();
                RawValue::Data(d.descriptor().identifier().into(), data)
            }
        }
    }
}

impl TryInto<CommonValue> for RawValue {
    type Error = ();

    fn try_into(self) -> Result<CommonValue, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<CommonValue> for &RawValue {
    type Error = ();

    fn try_into(self) -> Result<CommonValue, Self::Error> {
        match self {
            RawValue::Void(_) => Ok(CommonValue::Void(())),
            RawValue::I8(n) => Ok(CommonValue::I8(*n)),
            RawValue::I16(n) => Ok(CommonValue::I16(*n)),
            RawValue::I32(n) => Ok(CommonValue::I32(*n)),
            RawValue::I64(n) => Ok(CommonValue::I64(*n)),
            RawValue::I128(n) => Ok(CommonValue::I128(*n)),
            RawValue::U8(n) => Ok(CommonValue::U8(*n)),
            RawValue::U16(n) => Ok(CommonValue::U16(*n)),
            RawValue::U32(n) => Ok(CommonValue::U32(*n)),
            RawValue::U64(n) => Ok(CommonValue::U64(*n)),
            RawValue::U128(n) => Ok(CommonValue::U128(*n)),
            RawValue::F32(n) => Ok(CommonValue::F32(*n)),
            RawValue::F64(n) => Ok(CommonValue::F64(*n)),
            RawValue::Bool(b) => Ok(CommonValue::Bool(*b)),
            RawValue::Byte(b) => Ok(CommonValue::Byte(*b)),
            RawValue::Char(c) => Ok(CommonValue::Char(*c)),
            RawValue::String(s) => Ok(CommonValue::String(s.clone())),
            RawValue::Vec(v) => Ok({
                let mut vec = Vec::with_capacity(v.len());
                for val in v {
                    vec.push(val.try_into()?);
                }
                CommonValue::Vec(vec)
            }),
            RawValue::Option(v) => {
                if let Some(val) = v {
                    Ok(CommonValue::Option(Some(Box::new(
                        val.as_ref().try_into()?,
                    ))))
                } else {
                    Ok(CommonValue::Option(None))
                }
            }
            RawValue::Bytes(v) => Ok(CommonValue::Packed(CommonPackedArray::Byte(
                Arc::from(v.clone()),
            ))),
            RawValue::PackedI8(v) => Ok(CommonValue::Packed(CommonPackedArray::I8(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedI16(v) => Ok(CommonValue::Packed(CommonPackedArray::I16(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedI32(v) => Ok(CommonValue::Packed(CommonPackedArray::I32(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedI64(v) => Ok(CommonValue::Packed(CommonPackedArray::I64(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedI128(v) => Ok(CommonValue::Packed(CommonPackedArray::I128(
                Arc::from(v.clone()),
            ))),
            RawValue::PackedU8(v) => Ok(CommonValue::Packed(CommonPackedArray::U8(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedU16(v) => Ok(CommonValue::Packed(CommonPackedArray::U16(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedU32(v) => Ok(CommonValue::Packed(CommonPackedArray::U32(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedU64(v) => Ok(CommonValue::Packed(CommonPackedArray::U64(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedU128(v) => Ok(CommonValue::Packed(CommonPackedArray::U128(
                Arc::from(v.clone()),
            ))),
            RawValue::PackedF32(v) => Ok(CommonValue::Packed(CommonPackedArray::F32(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedF64(v) => Ok(CommonValue::Packed(CommonPackedArray::F64(Arc::from(
                v.clone(),
            )))),
            RawValue::PackedBool(v) => Ok(CommonValue::Packed(CommonPackedArray::Bool(
                Arc::from(v.clone()),
            ))),
            RawValue::PackedChar(v) => Ok(CommonValue::Packed(CommonPackedArray::Char(
                Arc::from(v.clone()),
            ))),
            RawValue::Data(_, _) => Err(()),
        }
    }
}

#[cfg(test)]
mod estimated_size_tests {
    use super::*;

    #[test]
    fn scalar_costs_only_the_enum_footprint() {
        let base = std::mem::size_of::<RawValue>();
        assert_eq!(RawValue::U64(42).estimated_size(), base);
    }

    #[test]
    fn string_costs_enum_footprint_plus_its_bytes() {
        let base = std::mem::size_of::<RawValue>();
        let text = "hello world".to_string();
        assert_eq!(
            RawValue::String(text.clone()).estimated_size(),
            base + text.len()
        );
    }

    #[test]
    fn vec_sums_enum_footprint_of_every_element() {
        let base = std::mem::size_of::<RawValue>();
        let value = RawValue::Vec(vec![
            RawValue::Byte(1),
            RawValue::Byte(2),
            RawValue::Byte(3),
        ]);
        assert_eq!(value.estimated_size(), base + 3 * base);
    }

    #[test]
    fn data_uses_its_actual_serialized_length_not_an_estimate() {
        let base = std::mem::size_of::<RawValue>();
        let identifier = Identifier {
            version: None,
            path: vec!["root".to_string()],
            name: "Name".to_string(),
        };
        let value = RawValue::Data(identifier, Some(vec![0u8; 42]));
        assert_eq!(value.estimated_size(), base + 42);
    }

    #[test]
    fn packed_bytes_has_no_per_element_overhead() {
        let base = std::mem::size_of::<RawValue>();
        let value = RawValue::Bytes(vec![0u8; 4096]);
        assert_eq!(value.estimated_size(), base + 4096);
    }

    #[test]
    fn packed_fixed_size_scalar_scales_with_type_size() {
        let base = std::mem::size_of::<RawValue>();
        let value = RawValue::PackedF64(vec![0f64; 10]);
        assert_eq!(value.estimated_size(), base + 10 * std::mem::size_of::<f64>());
    }
}

#[cfg(test)]
mod packed_wire_tests {
    use super::*;

    // The entire point of `RawValue::Bytes`: it must encode as CBOR's native byte
    // string (major type 2), not as an array of per-element-tagged integers like
    // `RawValue::Vec(Vec<RawValue::Byte>)` or even `RawValue::PackedU8` would. A byte
    // string's header is a single length-prefixed tag for the whole buffer, so the
    // encoded size should track the payload almost exactly, not scale with a
    // per-element tax.
    #[test]
    fn bytes_encodes_as_a_native_cbor_byte_string_not_a_tagged_array() {
        let payload = vec![0u8; 1000];
        let encoded = cbor4ii::serde::to_vec(Vec::new(), &RawValue::Bytes(payload.clone()))
            .expect("serialization must succeed");

        // A CBOR array of 1000 individually-tagged small integers would run to several
        // thousand bytes (1-2 bytes of header per element); a native byte string of
        // 1000 zero bytes is the payload plus a handful of header/enum-tag bytes.
        assert!(
            encoded.len() < payload.len() + 32,
            "expected near-payload-sized encoding for a native byte string, got {} bytes for a {}-byte payload",
            encoded.len(),
            payload.len()
        );

        let decoded: RawValue = cbor4ii::serde::from_slice(&encoded).expect("must round-trip");
        assert_eq!(decoded, RawValue::Bytes(payload));
    }

    // Non-byte packed types intentionally do NOT get the custom-encoding treatment —
    // they're a plain CBOR array of the native type, which already collapses the
    // per-element `RawValue` enum tag (the dominant cost) without needing a bespoke
    // binary layout. This just confirms the roundtrip holds for one such type.
    #[test]
    fn packed_f64_roundtrips_as_a_plain_cbor_array() {
        let payload = vec![1.5f64, 2.5, 3.5];
        let encoded = cbor4ii::serde::to_vec(Vec::new(), &RawValue::PackedF64(payload.clone()))
            .expect("serialization must succeed");
        let decoded: RawValue = cbor4ii::serde::from_slice(&encoded).expect("must round-trip");
        assert_eq!(decoded, RawValue::PackedF64(payload));
    }

    #[test]
    fn common_value_packed_byte_converts_to_raw_value_bytes() {
        let common = CommonValue::Packed(CommonPackedArray::Byte(Arc::from(vec![1u8, 2, 3])));
        let raw: RawValue = common.into();
        assert_eq!(raw, RawValue::Bytes(vec![1, 2, 3]));
    }

    #[test]
    fn common_value_packed_i64_converts_to_raw_value_packed_i64() {
        let common = CommonValue::Packed(CommonPackedArray::I64(Arc::from(vec![1i64, 2, 3])));
        let raw: RawValue = common.into();
        assert_eq!(raw, RawValue::PackedI64(vec![1, 2, 3]));
    }

    #[test]
    fn raw_value_bytes_converts_back_to_common_value_packed_byte() {
        let raw = RawValue::Bytes(vec![1u8, 2, 3]);
        let common: CommonValue = raw.try_into().unwrap();
        assert_eq!(
            common,
            CommonValue::Packed(CommonPackedArray::Byte(Arc::from(vec![1u8, 2, 3])))
        );
    }
}
