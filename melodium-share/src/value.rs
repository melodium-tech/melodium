use crate::{DataType, DescribedType, Identifier, SecretPolicy, SharingError, SharingResult};
use cbor4ii::core::utils::SliceReader;
use melodium_common::{
    descriptor::{Collection, Entry as CommonEntry, Identifier as CommonIdentifier},
    executive::{
        wipe_value, Secret as CommonSecret, SecretError, SecretOrigin, SecretTransfer,
        SecretsAccess, Value as CommonValue,
    },
};
use melodium_engine::{design::Value as DesignedValue, LogicError};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    pin::Pin,
    sync::Arc,
};
use zeroize::Zeroize;

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
            // A design keeps secrets as locators, never as values.
            Value::Raw(RawValue::Secret {
                name,
                datatype,
                policy,
                locator,
                value,
            }) => {
                let secret = match (datatype.to_datatype(collection), locator, value) {
                    (Some(datatype), Some(locator), None) => CommonSecret::new(
                        name.clone(),
                        datatype,
                        policy.into(),
                        SecretOrigin::Locator(locator.clone()),
                    )
                    .ok(),
                    _ => None,
                };
                if let Some(secret) = secret {
                    SharingResult::new_success(DesignedValue::Raw(CommonValue::Secret(secret)))
                } else {
                    SharingResult::new_failure(SharingError::data_serialization_error(19))
                }
            }
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

    /// Description of a secret, carrying its value only when sent by value
    /// to a distant engine (see `RawValue::to_wire`).
    ///
    /// `datatype` is the type of the value held (`T` for a `Secret<T>`),
    /// and `locator` is set when the value is resolved from a source.
    Secret {
        name: String,
        datatype: DataType,
        policy: SecretPolicy,
        locator: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<SecretValue>,
    },

    Data(Identifier, Option<Vec<u8>>),
}

/// Value of a secret sent by value to a distant engine, that `Debug` never shows,
/// overwritten when dropped (best effort, as for `melodium_common::executive::wipe_value`).
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
pub struct SecretValue(pub Box<RawValue>);

impl std::fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<secret value>")
    }
}

impl Drop for SecretValue {
    fn drop(&mut self) {
        self.0.wipe();
    }
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
            // Secrets are rebuilt by `from_wire`, the policy-aware conversion of the distribution layer.
            RawValue::Secret { .. } => None,
            other => other.try_into().ok(),
        }
    }

    /// Converts a value to send to a distant engine, each secret following its transmission
    /// policy (see `Secret::transmit`) for the element designated by `access`.
    ///
    /// Secrets sent by reference only carry their locator, and those sent by value carry
    /// their resolved value. Fails on the first secret refused.
    pub fn to_wire<'a>(
        value: &'a CommonValue,
        access: &'a SecretsAccess,
        encrypted: bool,
    ) -> Pin<Box<dyn Future<Output = Result<RawValue, SecretError>> + Send + 'a>> {
        Box::pin(async move {
            match value {
                CommonValue::Secret(secret) => {
                    let transfer = access.transmit(secret, encrypted).await?;
                    let mut raw: RawValue = secret.into();
                    if let RawValue::Secret { locator, value, .. } = &mut raw {
                        match transfer {
                            SecretTransfer::Reference(reference) => *locator = Some(reference),
                            SecretTransfer::Value(mut plain) => {
                                *locator = None;
                                *value = Some(SecretValue(Box::new((&plain).into())));
                                wipe_value(&mut plain);
                            }
                        }
                    }
                    Ok(raw)
                }
                CommonValue::Vec(values) if value.contains_secret() => {
                    let mut raw = Vec::with_capacity(values.len());
                    for value in values {
                        raw.push(Self::to_wire(value, access, encrypted).await?);
                    }
                    Ok(RawValue::Vec(raw))
                }
                CommonValue::Option(Some(inner)) if inner.contains_secret() => {
                    Ok(RawValue::Option(Some(Box::new(
                        Self::to_wire(inner, access, encrypted).await?,
                    ))))
                }
                other => Ok(other.into()),
            }
        })
    }

    /// Converts a value received from a distant engine.
    ///
    /// Secrets keep the policy they were sent with: those sent by reference resolve their
    /// locator with the sources of this engine, and those sent by value hold it inline.
    pub fn from_wire(&self, collection: &Collection) -> Option<CommonValue> {
        match self {
            RawValue::Secret {
                name,
                datatype,
                policy,
                locator,
                value,
            } => {
                let origin = match (value, locator) {
                    (Some(SecretValue(value)), _) => {
                        SecretOrigin::Inline(value.from_wire(collection)?)
                    }
                    (None, Some(locator)) => SecretOrigin::Locator(locator.clone()),
                    (None, None) => return None,
                };
                CommonSecret::new(
                    name.clone(),
                    datatype.to_datatype(collection)?,
                    policy.into(),
                    origin,
                )
                .ok()
                .map(CommonValue::Secret)
            }
            RawValue::Vec(v) => Some({
                let mut vec = Vec::with_capacity(v.len());
                for val in v {
                    vec.push(val.from_wire(collection)?);
                }
                CommonValue::Vec(vec)
            }),
            RawValue::Option(Some(value)) => Some(CommonValue::Option(Some(Box::new(
                value.from_wire(collection)?,
            )))),
            other => other.to_value(collection),
        }
    }

    /// Overwrites what this value holds, keeping its shape.
    fn wipe(&mut self) {
        match self {
            RawValue::Void(_) => {}
            RawValue::I8(n) => n.zeroize(),
            RawValue::I16(n) => n.zeroize(),
            RawValue::I32(n) => n.zeroize(),
            RawValue::I64(n) => n.zeroize(),
            RawValue::I128(n) => n.zeroize(),
            RawValue::U8(n) => n.zeroize(),
            RawValue::U16(n) => n.zeroize(),
            RawValue::U32(n) => n.zeroize(),
            RawValue::U64(n) => n.zeroize(),
            RawValue::U128(n) => n.zeroize(),
            RawValue::F32(n) => n.zeroize(),
            RawValue::F64(n) => n.zeroize(),
            RawValue::Bool(b) => b.zeroize(),
            RawValue::Byte(b) => b.zeroize(),
            RawValue::Char(c) => c.zeroize(),
            RawValue::String(s) => s.zeroize(),
            RawValue::Vec(values) => values.iter_mut().for_each(RawValue::wipe),
            RawValue::Option(value) => {
                if let Some(value) = value {
                    value.wipe();
                }
            }
            // Its value, if any, is wiped when dropped.
            RawValue::Secret { .. } => {}
            RawValue::Data(_, data) => data.zeroize(),
        }
    }

    /// Gives this value without the values of secrets sent by value,
    /// for what is kept or shown beyond the transport itself.
    pub fn without_secret_values(&self) -> RawValue {
        match self {
            RawValue::Secret {
                name,
                datatype,
                policy,
                locator,
                value: _,
            } => RawValue::Secret {
                name: name.clone(),
                datatype: datatype.clone(),
                policy: policy.clone(),
                locator: locator.clone(),
                value: None,
            },
            RawValue::Vec(values) => {
                RawValue::Vec(values.iter().map(RawValue::without_secret_values).collect())
            }
            RawValue::Option(Some(value)) => {
                RawValue::Option(Some(Box::new(value.without_secret_values())))
            }
            other => other.clone(),
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
                RawValue::Secret {
                    name,
                    locator,
                    value,
                    ..
                } => {
                    name.len()
                        + locator.as_ref().map(String::len).unwrap_or(0)
                        + value
                            .as_ref()
                            .map(|SecretValue(value)| value.estimated_size())
                            .unwrap_or(0)
                }
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
            // `RawValue` intentionally stays canonical — one variant per mel scalar
            // type, `Vec`/`Option`/`Data` for aggregates, nothing else. It's exposed
            // directly over WASM (`tsify`, `into_wasm_abi`/`from_wasm_abi` below) and
            // used for saved program designs, so it's a public contract, not an
            // internal wire-optimization detail: `Packed` is purely an in-process
            // representation choice (see `PackedArray`) that should never leak into
            // what an external consumer has to pattern-match on. A packed array
            // expands back to one `RawValue` per element here, exactly what
            // `Value::Vec` would have produced for the same content.
            CommonValue::Packed(arr) => RawValue::Vec(
                arr.into_values()
                    .into_iter()
                    .map(|value| value.into())
                    .collect(),
            ),
            CommonValue::Secret(secret) => (&secret).into(),
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
            // `RawValue` intentionally stays canonical — one variant per mel scalar
            // type, `Vec`/`Option`/`Data` for aggregates, nothing else. It's exposed
            // directly over WASM (`tsify`, `into_wasm_abi`/`from_wasm_abi` below) and
            // used for saved program designs, so it's a public contract, not an
            // internal wire-optimization detail: `Packed` is purely an in-process
            // representation choice (see `PackedArray`) that should never leak into
            // what an external consumer has to pattern-match on. A packed array
            // expands back to one `RawValue` per element here, exactly what
            // `Value::Vec` would have produced for the same content.
            CommonValue::Packed(arr) => RawValue::Vec(
                arr.clone()
                    .into_values()
                    .into_iter()
                    .map(|value| value.into())
                    .collect(),
            ),
            CommonValue::Secret(secret) => secret.into(),
            CommonValue::Data(d) => {
                let data = cbor4ii::serde::to_vec(Vec::new(), &d).ok();
                RawValue::Data(d.descriptor().identifier().into(), data)
            }
        }
    }
}

impl From<&CommonSecret> for RawValue {
    fn from(secret: &CommonSecret) -> Self {
        RawValue::Secret {
            name: secret.name().to_string(),
            datatype: secret.datatype().into(),
            policy: secret.policy().into(),
            locator: secret.locator().map(str::to_string),
            value: None,
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
            RawValue::Secret { .. } => Err(()),
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

#[cfg(test)]
mod packed_value_stays_canonical_tests {
    use super::*;
    use melodium_common::executive::PackedArray as CommonPackedArray;

    // `RawValue` deliberately has no `Packed` counterpart (see the comment on the
    // `CommonValue::Packed` match arm above) — a packed in-process `Value` must still
    // convert to the exact same `RawValue::Vec` shape a boxed `Value::Vec` would,
    // so nothing external (WASM bindings, saved designs, ...) can tell the two apart.
    #[test]
    fn common_value_packed_converts_to_the_same_raw_shape_as_the_boxed_equivalent() {
        let packed = CommonValue::Packed(CommonPackedArray::Byte(Arc::new(vec![1u8, 2, 3])));
        let boxed = CommonValue::Vec(vec![
            CommonValue::Byte(1),
            CommonValue::Byte(2),
            CommonValue::Byte(3),
        ]);

        let raw_from_packed: RawValue = packed.into();
        let raw_from_boxed: RawValue = boxed.into();
        assert_eq!(raw_from_packed, raw_from_boxed);
        assert_eq!(
            raw_from_packed,
            RawValue::Vec(vec![
                RawValue::Byte(1),
                RawValue::Byte(2),
                RawValue::Byte(3)
            ])
        );
    }

    #[test]
    fn common_value_packed_roundtrips_through_raw_value_as_a_boxed_vec() {
        let packed = CommonValue::Packed(CommonPackedArray::I64(Arc::new(vec![1, 2, 3])));
        let raw: RawValue = packed.into();
        let back: CommonValue = raw.try_into().unwrap();
        assert_eq!(
            back,
            CommonValue::Vec(vec![
                CommonValue::I64(1),
                CommonValue::I64(2),
                CommonValue::I64(3)
            ])
        );
    }
}

#[cfg(test)]
mod secret_tests {
    use super::*;
    use crate::{SecretReveal, SecretTransmission};
    use melodium_common::{
        descriptor::DataType as CommonDataType,
        executive::{SecretOrigin, SecretPolicy as CommonSecretPolicy},
    };

    const SENTINEL: &str = "s3cr3t-sentinel-value";

    fn secret(origin: SecretOrigin) -> CommonValue {
        CommonValue::Secret(
            CommonSecret::new(
                "db_password".to_string(),
                CommonDataType::String,
                CommonSecretPolicy::default(),
                origin,
            )
            .unwrap(),
        )
    }

    fn assert_no_sentinel(raw: &RawValue) {
        let debug = format!("{raw:?}");
        assert!(!debug.contains(SENTINEL), "leaked in {debug}");
        let cbor = cbor4ii::serde::to_vec(Vec::new(), raw).unwrap();
        assert!(!cbor
            .windows(SENTINEL.len())
            .any(|window| window == SENTINEL.as_bytes()));
    }

    #[test]
    fn inline_secret_converts_without_its_value() {
        let value = secret(SecretOrigin::Inline(CommonValue::String(
            SENTINEL.to_string(),
        )));
        let expected = RawValue::Secret {
            name: "db_password".to_string(),
            datatype: DataType::String,
            policy: SecretPolicy {
                transmission: SecretTransmission::Local,
                reveal: SecretReveal::Any,
                plain_reveal: false,
            },
            locator: None,
            value: None,
        };

        let by_ref: RawValue = (&value).into();
        let by_value: RawValue = value.clone().into();
        assert_eq!(by_ref, expected);
        assert_eq!(by_value, expected);

        for raw in [
            by_ref,
            CommonValue::Vec(vec![value.clone()]).into(),
            CommonValue::Option(Some(Box::new(value))).into(),
        ] {
            assert_no_sentinel(&raw);
        }
    }

    #[test]
    fn locator_secret_keeps_its_locator() {
        let raw: RawValue = secret(SecretOrigin::Locator("env:DB_PASSWORD".to_string())).into();
        assert!(matches!(
            raw,
            RawValue::Secret { locator: Some(ref locator), .. } if locator == "env:DB_PASSWORD"
        ));
    }

    #[test]
    fn raw_secret_never_converts_back_to_a_value() {
        let raw: RawValue = secret(SecretOrigin::Inline(CommonValue::String(
            SENTINEL.to_string(),
        )))
        .into();
        let collection = Collection::new();
        assert!(raw.to_value(&collection).is_none());
        assert!(RawValue::Vec(vec![raw.clone()])
            .to_value(&collection)
            .is_none());
        assert!(TryInto::<CommonValue>::try_into(&raw).is_err());
    }

    #[test]
    fn designs_keep_locator_secrets_only() {
        let collection = Collection::new();
        let scope = CommonIdentifier::new(vec!["root".to_string()], "Scope");

        let locator: RawValue = secret(SecretOrigin::Locator("env:DB_PASSWORD".to_string())).into();
        match Value::Raw(locator).to_value(&collection, &scope).success() {
            Some(DesignedValue::Raw(CommonValue::Secret(secret))) => {
                assert_eq!(secret.name(), "db_password");
                assert_eq!(secret.locator(), Some("env:DB_PASSWORD"));
                assert_eq!(secret.datatype(), &CommonDataType::String);
            }
            other => panic!("locator secret expected, got {other:?}"),
        }

        let inline: RawValue = secret(SecretOrigin::Inline(CommonValue::String(
            SENTINEL.to_string(),
        )))
        .into();
        assert!(Value::Raw(inline)
            .to_value(&collection, &scope)
            .is_failure());
    }

    #[test]
    fn secret_size_does_not_depend_on_inline_value() {
        let short: RawValue =
            secret(SecretOrigin::Inline(CommonValue::String("a".to_string()))).into();
        let long: RawValue =
            secret(SecretOrigin::Inline(CommonValue::String("a".repeat(4096)))).into();
        assert_eq!(short.estimated_size(), long.estimated_size());
    }
}

#[cfg(test)]
mod secret_wire_tests {
    use super::*;
    use crate::SecretTransmission;
    use async_std::task::block_on;
    use async_trait::async_trait;
    use melodium_common::executive::{SecretAudit, SecretSource, SecretsHost};
    use melodium_common::{
        descriptor::DataType as CommonDataType,
        executive::{
            SecretPolicy as CommonSecretPolicy, SecretTransmission as CommonSecretTransmission,
            TransmissionValue as CommonTransmissionValue,
        },
    };

    const SENTINEL: &str = "s3cr3t-wire-sentinel";

    #[derive(Debug)]
    struct EnvironmentSource;

    #[async_trait]
    impl SecretSource for EnvironmentSource {
        async fn resolve(
            &self,
            path: &str,
            _datatype: &CommonDataType,
        ) -> Result<CommonValue, String> {
            std::env::var(path)
                .map(CommonValue::String)
                .map_err(|_| format!("environment variable '{path}' is not set"))
        }
    }

    /// Secrets host of a started engine, with an `env:` source.
    #[derive(Debug)]
    struct Host;

    #[async_trait]
    impl SecretsHost for Host {
        fn secret_source(&self, scheme: &str) -> Option<Arc<dyn SecretSource>> {
            (scheme == "env").then(|| Arc::new(EnvironmentSource) as Arc<dyn SecretSource>)
        }

        fn register_secret_source(
            &self,
            _scheme: &str,
            _source: Arc<dyn SecretSource>,
        ) -> Result<(), SecretError> {
            Err(SecretError::RegistrationClosed)
        }

        fn revealing(&self) -> bool {
            true
        }

        fn add_masked_value(&self, _secret_name: &str, _value: &CommonValue) {}

        async fn secret_audit(&self, _audit: SecretAudit) {}
    }

    fn host() -> Arc<dyn SecretsHost> {
        Arc::new(Host)
    }

    fn access(host: &Arc<dyn SecretsHost>) -> SecretsAccess {
        SecretsAccess::new(
            host,
            CommonIdentifier::new(vec!["test".to_string()], "sender"),
            None,
            None,
        )
    }

    fn secret(transmission: CommonSecretTransmission, origin: SecretOrigin) -> CommonValue {
        CommonValue::Secret(
            CommonSecret::new(
                "token".to_string(),
                CommonDataType::String,
                CommonSecretPolicy {
                    transmission,
                    ..CommonSecretPolicy::default()
                },
                origin,
            )
            .unwrap(),
        )
    }

    fn inline(transmission: CommonSecretTransmission) -> CommonValue {
        secret(
            transmission,
            SecretOrigin::Inline(CommonValue::String(SENTINEL.to_string())),
        )
    }

    fn contains_sentinel(raw: &RawValue) -> bool {
        let cbor = cbor4ii::serde::to_vec(Vec::new(), raw).unwrap();
        cbor.windows(SENTINEL.len())
            .any(|window| window == SENTINEL.as_bytes())
    }

    #[test]
    fn local_secrets_are_refused() {
        let host = host();
        let access = access(&host);
        for encrypted in [false, true] {
            assert!(matches!(
                block_on(RawValue::to_wire(
                    &inline(CommonSecretTransmission::Local),
                    &access,
                    encrypted
                )),
                Err(SecretError::Denied(_))
            ));
        }
    }

    #[test]
    fn references_carry_the_locator_only() {
        let host = host();
        let access = access(&host);
        // Never resolved by the sender, so the variable does not need to exist.
        let value = secret(
            CommonSecretTransmission::Reference,
            SecretOrigin::Locator("env:MELODIUM_SECRET_SHARE_WIRE_TEST_UNSET".to_string()),
        );
        let raw = block_on(RawValue::to_wire(&value, &access, false)).unwrap();
        assert!(matches!(
            &raw,
            RawValue::Secret { locator: Some(locator), value: None, policy, .. }
                if locator == "env:MELODIUM_SECRET_SHARE_WIRE_TEST_UNSET"
                    && policy.transmission == SecretTransmission::Reference
        ));

        match raw.from_wire(&Collection::new()) {
            Some(CommonValue::Secret(received)) => {
                assert_eq!(received.name(), "token");
                assert_eq!(
                    received.locator(),
                    Some("env:MELODIUM_SECRET_SHARE_WIRE_TEST_UNSET")
                );
                assert_eq!(
                    received.policy().transmission,
                    CommonSecretTransmission::Reference
                );
            }
            other => panic!("secret expected, got {other:?}"),
        }

        assert!(matches!(
            block_on(RawValue::to_wire(
                &inline(CommonSecretTransmission::Reference),
                &access,
                true
            )),
            Err(SecretError::Denied(_))
        ));
    }

    #[test]
    fn values_cross_encrypted_connections_only() {
        let host = host();
        let access = access(&host);
        let value = inline(CommonSecretTransmission::Value);

        assert!(matches!(
            block_on(RawValue::to_wire(&value, &access, false)),
            Err(SecretError::Denied(_))
        ));

        let raw = block_on(RawValue::to_wire(&value, &access, true)).unwrap();
        assert!(matches!(
            &raw,
            RawValue::Secret {
                locator: None,
                value: Some(_),
                ..
            }
        ));
        assert!(contains_sentinel(&raw));
        assert!(!format!("{raw:?}").contains(SENTINEL));
        assert!(!contains_sentinel(&raw.without_secret_values()));

        let received = raw.from_wire(&Collection::new()).unwrap();
        let back = block_on(RawValue::to_wire(&received, &access, true)).unwrap();
        assert_eq!(back, raw);
    }

    #[test]
    fn values_are_resolved_by_the_sender() {
        let host = host();
        let access = access(&host);
        std::env::set_var("MELODIUM_SECRET_SHARE_WIRE_TEST_VALUE", SENTINEL);
        let value = secret(
            CommonSecretTransmission::Value,
            SecretOrigin::Locator("env:MELODIUM_SECRET_SHARE_WIRE_TEST_VALUE".to_string()),
        );
        let raw = block_on(RawValue::to_wire(&value, &access, true)).unwrap();
        assert!(matches!(
            &raw,
            RawValue::Secret {
                locator: None,
                value: Some(SecretValue(value)),
                ..
            } if **value == RawValue::String(SENTINEL.to_string())
        ));
    }

    #[test]
    fn containers_and_batches_carry_secrets() {
        let host = host();
        let access = access(&host);
        let collection = Collection::new();

        let vec = CommonValue::Vec(vec![
            inline(CommonSecretTransmission::Value),
            inline(CommonSecretTransmission::Value),
        ]);
        let raw = block_on(RawValue::to_wire(&vec, &access, true)).unwrap();
        assert!(contains_sentinel(&raw));
        assert!(matches!(
            raw.from_wire(&collection),
            Some(CommonValue::Vec(values)) if values.len() == 2 && values.iter().all(|value| matches!(value, CommonValue::Secret(_)))
        ));

        let option = CommonValue::Option(Some(Box::new(inline(CommonSecretTransmission::Local))));
        assert!(block_on(RawValue::to_wire(&option, &access, true)).is_err());

        let batch = CommonTransmissionValue::Other(
            vec![
                inline(CommonSecretTransmission::Value),
                inline(CommonSecretTransmission::Value),
            ]
            .into(),
        );
        let wire = block_on(crate::TransmissionValue::to_wire(batch, &access, true)).unwrap();
        let received = wire.to_transmission_value(&collection).unwrap();
        assert_eq!(received.len(), 2);

        let plain =
            CommonTransmissionValue::Other(vec![CommonValue::String("a".to_string())].into());
        assert_eq!(
            block_on(crate::TransmissionValue::to_wire(
                plain.clone(),
                &access,
                false
            ))
            .unwrap(),
            plain.into()
        );
    }

    #[test]
    fn wiping_overwrites_values_and_keeps_their_shape() {
        let mut raw = RawValue::Vec(vec![
            RawValue::String(SENTINEL.to_string()),
            RawValue::U64(42),
            RawValue::Option(Some(Box::new(RawValue::Char('s')))),
        ]);
        raw.wipe();
        assert_eq!(
            raw,
            RawValue::Vec(vec![
                RawValue::String(String::new()),
                RawValue::U64(0),
                RawValue::Option(Some(Box::new(RawValue::Char('\0')))),
            ])
        );
    }

    #[test]
    fn designs_refuse_secret_values() {
        let host = host();
        let access = access(&host);
        let raw = block_on(RawValue::to_wire(
            &inline(CommonSecretTransmission::Value),
            &access,
            true,
        ))
        .unwrap();
        let raw = match raw {
            RawValue::Secret {
                name,
                datatype,
                policy,
                value,
                ..
            } => RawValue::Secret {
                name,
                datatype,
                policy,
                locator: Some("env:MELODIUM_SECRET_SHARE_WIRE_TEST_UNSET".to_string()),
                value,
            },
            other => other,
        };
        let scope = CommonIdentifier::new(vec!["root".to_string()], "Scope");
        assert!(Value::Raw(raw)
            .to_value(&Collection::new(), &scope)
            .is_failure());
    }
}
