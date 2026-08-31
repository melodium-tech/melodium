use crate::{DescribedType, Identifier, SharingError, SharingResult};
use cbor4ii::core::utils::SliceReader;
use melodium_common::{
    descriptor::{Collection, Entry as CommonEntry, Identifier as CommonIdentifier},
    executive::Value as CommonValue,
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
}
