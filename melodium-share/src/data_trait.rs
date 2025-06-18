use melodium_common::descriptor::DataTrait as CommonDataTrait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum DataTrait {
    Option,
    Vec,

    // Infallilble conversions
    ToI8,
    ToI16,
    ToI32,
    ToI64,
    ToI128,
    ToU8,
    ToU16,
    ToU32,
    ToU64,
    ToU128,
    ToF32,
    ToF64,
    ToBool,
    ToByte,
    ToChar,
    ToString,

    // Faillible conversions (may give `none`)
    TryToI8,
    TryToI16,
    TryToI32,
    TryToI64,
    TryToI128,
    TryToU8,
    TryToU16,
    TryToU32,
    TryToU64,
    TryToU128,
    TryToF32,
    TryToF64,
    TryToBool,
    TryToByte,
    TryToChar,
    TryToString,

    SaturatingToI8,
    SaturatingToI16,
    SaturatingToI32,
    SaturatingToI64,
    SaturatingToI128,
    SaturatingToU8,
    SaturatingToU16,
    SaturatingToU32,
    SaturatingToU64,
    SaturatingToU128,
    SaturatingToF32,
    SaturatingToF64,

    Bounded,

    Binary,

    Signed,
    Float,

    PartialEquality,
    Equality,
    PartialOrder,
    Order,

    Add,
    CheckedAdd,
    SaturatingAdd,
    WrappingAdd,
    Sub,
    CheckedSub,
    SaturatingSub,
    WrappingSub,
    Mul,
    CheckedMul,
    SaturatingMul,
    WrappingMul,
    Div,
    CheckedDiv,
    Rem,
    CheckedRem,
    Neg,
    CheckedNeg,
    WrappingNeg,
    Pow,
    CheckedPow,

    Euclid,
    CheckedEuclid,

    Hash,

    Serialize,
    Deserialize,

    Display,
}

impl From<&CommonDataTrait> for DataTrait {
    fn from(value: &CommonDataTrait) -> Self {
        match value {
            CommonDataTrait::Option => DataTrait::Option,
            CommonDataTrait::Vec => DataTrait::Vec,

            CommonDataTrait::ToI8 => DataTrait::ToI8,
            CommonDataTrait::ToI16 => DataTrait::ToI16,
            CommonDataTrait::ToI32 => DataTrait::ToI32,
            CommonDataTrait::ToI64 => DataTrait::ToI64,
            CommonDataTrait::ToI128 => DataTrait::ToI128,
            CommonDataTrait::ToU8 => DataTrait::ToU8,
            CommonDataTrait::ToU16 => DataTrait::ToU16,
            CommonDataTrait::ToU32 => DataTrait::ToU32,
            CommonDataTrait::ToU64 => DataTrait::ToU64,
            CommonDataTrait::ToU128 => DataTrait::ToU128,
            CommonDataTrait::ToF32 => DataTrait::ToF32,
            CommonDataTrait::ToF64 => DataTrait::ToF64,
            CommonDataTrait::ToBool => DataTrait::ToBool,
            CommonDataTrait::ToByte => DataTrait::ToByte,
            CommonDataTrait::ToChar => DataTrait::ToChar,
            CommonDataTrait::ToString => DataTrait::ToString,

            CommonDataTrait::TryToI8 => DataTrait::TryToI8,
            CommonDataTrait::TryToI16 => DataTrait::TryToI16,
            CommonDataTrait::TryToI32 => DataTrait::TryToI32,
            CommonDataTrait::TryToI64 => DataTrait::TryToI64,
            CommonDataTrait::TryToI128 => DataTrait::TryToI128,
            CommonDataTrait::TryToU8 => DataTrait::TryToU8,
            CommonDataTrait::TryToU16 => DataTrait::TryToU16,
            CommonDataTrait::TryToU32 => DataTrait::TryToU32,
            CommonDataTrait::TryToU64 => DataTrait::TryToU64,
            CommonDataTrait::TryToU128 => DataTrait::TryToU128,
            CommonDataTrait::TryToF32 => DataTrait::TryToF32,
            CommonDataTrait::TryToF64 => DataTrait::TryToF64,
            CommonDataTrait::TryToBool => DataTrait::TryToBool,
            CommonDataTrait::TryToByte => DataTrait::TryToByte,
            CommonDataTrait::TryToChar => DataTrait::TryToChar,
            CommonDataTrait::TryToString => DataTrait::TryToString,

            CommonDataTrait::SaturatingToI8 => DataTrait::SaturatingToI8,
            CommonDataTrait::SaturatingToI16 => DataTrait::SaturatingToI16,
            CommonDataTrait::SaturatingToI32 => DataTrait::SaturatingToI32,
            CommonDataTrait::SaturatingToI64 => DataTrait::SaturatingToI64,
            CommonDataTrait::SaturatingToI128 => DataTrait::SaturatingToI128,
            CommonDataTrait::SaturatingToU8 => DataTrait::SaturatingToU8,
            CommonDataTrait::SaturatingToU16 => DataTrait::SaturatingToU16,
            CommonDataTrait::SaturatingToU32 => DataTrait::SaturatingToU32,
            CommonDataTrait::SaturatingToU64 => DataTrait::SaturatingToU64,
            CommonDataTrait::SaturatingToU128 => DataTrait::SaturatingToU128,
            CommonDataTrait::SaturatingToF32 => DataTrait::SaturatingToF32,
            CommonDataTrait::SaturatingToF64 => DataTrait::SaturatingToF64,

            CommonDataTrait::Bounded => DataTrait::Bounded,

            CommonDataTrait::Binary => DataTrait::Binary,

            CommonDataTrait::Signed => DataTrait::Signed,
            CommonDataTrait::Float => DataTrait::Float,

            CommonDataTrait::PartialEquality => DataTrait::PartialEquality,
            CommonDataTrait::Equality => DataTrait::Equality,
            CommonDataTrait::PartialOrder => DataTrait::PartialOrder,
            CommonDataTrait::Order => DataTrait::Order,

            CommonDataTrait::Add => DataTrait::Add,
            CommonDataTrait::CheckedAdd => DataTrait::CheckedAdd,
            CommonDataTrait::SaturatingAdd => DataTrait::SaturatingAdd,
            CommonDataTrait::WrappingAdd => DataTrait::WrappingAdd,
            CommonDataTrait::Sub => DataTrait::Sub,
            CommonDataTrait::CheckedSub => DataTrait::CheckedSub,
            CommonDataTrait::SaturatingSub => DataTrait::SaturatingSub,
            CommonDataTrait::WrappingSub => DataTrait::WrappingSub,
            CommonDataTrait::Mul => DataTrait::Mul,
            CommonDataTrait::CheckedMul => DataTrait::CheckedMul,
            CommonDataTrait::SaturatingMul => DataTrait::SaturatingMul,
            CommonDataTrait::WrappingMul => DataTrait::WrappingMul,
            CommonDataTrait::Div => DataTrait::Div,
            CommonDataTrait::CheckedDiv => DataTrait::CheckedDiv,
            CommonDataTrait::Rem => DataTrait::Rem,
            CommonDataTrait::CheckedRem => DataTrait::CheckedRem,
            CommonDataTrait::Neg => DataTrait::Neg,
            CommonDataTrait::CheckedNeg => DataTrait::CheckedNeg,
            CommonDataTrait::WrappingNeg => DataTrait::WrappingNeg,
            CommonDataTrait::Pow => DataTrait::Pow,
            CommonDataTrait::CheckedPow => DataTrait::CheckedPow,

            CommonDataTrait::Euclid => DataTrait::Euclid,
            CommonDataTrait::CheckedEuclid => DataTrait::CheckedEuclid,

            CommonDataTrait::Hash => DataTrait::Hash,

            CommonDataTrait::Serialize => DataTrait::Serialize,
            CommonDataTrait::Deserialize => DataTrait::Deserialize,

            CommonDataTrait::Display => DataTrait::Display,
        }
    }
}

impl Into<CommonDataTrait> for &DataTrait {
    fn into(self) -> CommonDataTrait {
        match self {
            DataTrait::Option => CommonDataTrait::Option,
            DataTrait::Vec => CommonDataTrait::Vec,

            DataTrait::ToI8 => CommonDataTrait::ToI8,
            DataTrait::ToI16 => CommonDataTrait::ToI16,
            DataTrait::ToI32 => CommonDataTrait::ToI32,
            DataTrait::ToI64 => CommonDataTrait::ToI64,
            DataTrait::ToI128 => CommonDataTrait::ToI128,
            DataTrait::ToU8 => CommonDataTrait::ToU8,
            DataTrait::ToU16 => CommonDataTrait::ToU16,
            DataTrait::ToU32 => CommonDataTrait::ToU32,
            DataTrait::ToU64 => CommonDataTrait::ToU64,
            DataTrait::ToU128 => CommonDataTrait::ToU128,
            DataTrait::ToF32 => CommonDataTrait::ToF32,
            DataTrait::ToF64 => CommonDataTrait::ToF64,
            DataTrait::ToBool => CommonDataTrait::ToBool,
            DataTrait::ToByte => CommonDataTrait::ToByte,
            DataTrait::ToChar => CommonDataTrait::ToChar,
            DataTrait::ToString => CommonDataTrait::ToString,

            DataTrait::TryToI8 => CommonDataTrait::TryToI8,
            DataTrait::TryToI16 => CommonDataTrait::TryToI16,
            DataTrait::TryToI32 => CommonDataTrait::TryToI32,
            DataTrait::TryToI64 => CommonDataTrait::TryToI64,
            DataTrait::TryToI128 => CommonDataTrait::TryToI128,
            DataTrait::TryToU8 => CommonDataTrait::TryToU8,
            DataTrait::TryToU16 => CommonDataTrait::TryToU16,
            DataTrait::TryToU32 => CommonDataTrait::TryToU32,
            DataTrait::TryToU64 => CommonDataTrait::TryToU64,
            DataTrait::TryToU128 => CommonDataTrait::TryToU128,
            DataTrait::TryToF32 => CommonDataTrait::TryToF32,
            DataTrait::TryToF64 => CommonDataTrait::TryToF64,
            DataTrait::TryToBool => CommonDataTrait::TryToBool,
            DataTrait::TryToByte => CommonDataTrait::TryToByte,
            DataTrait::TryToChar => CommonDataTrait::TryToChar,
            DataTrait::TryToString => CommonDataTrait::TryToString,

            DataTrait::SaturatingToI8 => CommonDataTrait::SaturatingToI8,
            DataTrait::SaturatingToI16 => CommonDataTrait::SaturatingToI16,
            DataTrait::SaturatingToI32 => CommonDataTrait::SaturatingToI32,
            DataTrait::SaturatingToI64 => CommonDataTrait::SaturatingToI64,
            DataTrait::SaturatingToI128 => CommonDataTrait::SaturatingToI128,
            DataTrait::SaturatingToU8 => CommonDataTrait::SaturatingToU8,
            DataTrait::SaturatingToU16 => CommonDataTrait::SaturatingToU16,
            DataTrait::SaturatingToU32 => CommonDataTrait::SaturatingToU32,
            DataTrait::SaturatingToU64 => CommonDataTrait::SaturatingToU64,
            DataTrait::SaturatingToU128 => CommonDataTrait::SaturatingToU128,
            DataTrait::SaturatingToF32 => CommonDataTrait::SaturatingToF32,
            DataTrait::SaturatingToF64 => CommonDataTrait::SaturatingToF64,

            DataTrait::Bounded => CommonDataTrait::Bounded,

            DataTrait::Binary => CommonDataTrait::Binary,

            DataTrait::Signed => CommonDataTrait::Signed,
            DataTrait::Float => CommonDataTrait::Float,

            DataTrait::PartialEquality => CommonDataTrait::PartialEquality,
            DataTrait::Equality => CommonDataTrait::Equality,
            DataTrait::PartialOrder => CommonDataTrait::PartialOrder,
            DataTrait::Order => CommonDataTrait::Order,

            DataTrait::Add => CommonDataTrait::Add,
            DataTrait::CheckedAdd => CommonDataTrait::CheckedAdd,
            DataTrait::SaturatingAdd => CommonDataTrait::SaturatingAdd,
            DataTrait::WrappingAdd => CommonDataTrait::WrappingAdd,
            DataTrait::Sub => CommonDataTrait::Sub,
            DataTrait::CheckedSub => CommonDataTrait::CheckedSub,
            DataTrait::SaturatingSub => CommonDataTrait::SaturatingSub,
            DataTrait::WrappingSub => CommonDataTrait::WrappingSub,
            DataTrait::Mul => CommonDataTrait::Mul,
            DataTrait::CheckedMul => CommonDataTrait::CheckedMul,
            DataTrait::SaturatingMul => CommonDataTrait::SaturatingMul,
            DataTrait::WrappingMul => CommonDataTrait::WrappingMul,
            DataTrait::Div => CommonDataTrait::Div,
            DataTrait::CheckedDiv => CommonDataTrait::CheckedDiv,
            DataTrait::Rem => CommonDataTrait::Rem,
            DataTrait::CheckedRem => CommonDataTrait::CheckedRem,
            DataTrait::Neg => CommonDataTrait::Neg,
            DataTrait::CheckedNeg => CommonDataTrait::CheckedNeg,
            DataTrait::WrappingNeg => CommonDataTrait::WrappingNeg,
            DataTrait::Pow => CommonDataTrait::Pow,
            DataTrait::CheckedPow => CommonDataTrait::CheckedPow,

            DataTrait::Euclid => CommonDataTrait::Euclid,
            DataTrait::CheckedEuclid => CommonDataTrait::CheckedEuclid,

            DataTrait::Hash => CommonDataTrait::Hash,

            DataTrait::Serialize => CommonDataTrait::Serialize,
            DataTrait::Deserialize => CommonDataTrait::Deserialize,

            DataTrait::Display => CommonDataTrait::Display,
        }
    }
}

impl Into<CommonDataTrait> for DataTrait {
    fn into(self) -> CommonDataTrait {
        (&self).into()
    }
}
