use crate::string;

pub trait ToI8 {
    fn to_i8(&self) -> i8;
}

pub trait ToI16 {
    fn to_i16(&self) -> i16;
}

pub trait ToI32 {
    fn to_i32(&self) -> i32;
}

pub trait ToI64 {
    fn to_i64(&self) -> i64;
}

pub trait ToI128 {
    fn to_i128(&self) -> i128;
}

pub trait ToU8 {
    fn to_u8(&self) -> u8;
}

pub trait ToU16 {
    fn to_u16(&self) -> u16;
}

pub trait ToU32 {
    fn to_u32(&self) -> u32;
}

pub trait ToU64 {
    fn to_u64(&self) -> u64;
}

pub trait ToU128 {
    fn to_u128(&self) -> u128;
}

pub trait ToF32 {
    fn to_f32(&self) -> f32;
}

pub trait ToF64 {
    fn to_f64(&self) -> f64;
}

pub trait ToBool {
    fn to_bool(&self) -> bool;
}

pub trait ToByte {
    fn to_byte(&self) -> u8;
}

pub trait ToChar {
    fn to_char(&self) -> char;
}

pub trait ToString {
    fn to_string(&self) -> string;
}

pub trait TryToI8 {
    fn try_to_i8(&self) -> Option<i8>;
}

pub trait TryToI16 {
    fn try_to_i16(&self) -> Option<i16>;
}

pub trait TryToI32 {
    fn try_to_i32(&self) -> Option<i32>;
}

pub trait TryToI64 {
    fn try_to_i64(&self) -> Option<i64>;
}

pub trait TryToI128 {
    fn try_to_i128(&self) -> Option<i128>;
}

pub trait TryToU8 {
    fn try_to_u8(&self) -> Option<u8>;
}

pub trait TryToU16 {
    fn try_to_u16(&self) -> Option<u16>;
}

pub trait TryToU32 {
    fn try_to_u32(&self) -> Option<u32>;
}

pub trait TryToU64 {
    fn try_to_u64(&self) -> Option<u64>;
}

pub trait TryToU128 {
    fn try_to_u128(&self) -> Option<u128>;
}

pub trait TryToF32 {
    fn try_to_f32(&self) -> Option<f32>;
}

pub trait TryToF64 {
    fn try_to_f64(&self) -> Option<f64>;
}

pub trait TryToBool {
    fn try_to_bool(&self) -> Option<bool>;
}

pub trait TryToByte {
    fn try_to_byte(&self) -> Option<u8>;
}

pub trait TryToChar {
    fn try_to_char(&self) -> Option<char>;
}

pub trait TryToString {
    fn try_to_string(&self) -> Option<string>;
}

pub trait SaturatingToI8 {
    fn saturating_to_i8(&self) -> i8;
}

pub trait SaturatingToI16 {
    fn saturating_to_i16(&self) -> i16;
}

pub trait SaturatingToI32 {
    fn saturating_to_i32(&self) -> i32;
}

pub trait SaturatingToI64 {
    fn saturating_to_i64(&self) -> i64;
}

pub trait SaturatingToI128 {
    fn saturating_to_i128(&self) -> i128;
}

pub trait SaturatingToU8 {
    fn saturating_to_u8(&self) -> u8;
}

pub trait SaturatingToU16 {
    fn saturating_to_u16(&self) -> u16;
}

pub trait SaturatingToU32 {
    fn saturating_to_u32(&self) -> u32;
}

pub trait SaturatingToU64 {
    fn saturating_to_u64(&self) -> u64;
}

pub trait SaturatingToU128 {
    fn saturating_to_u128(&self) -> u128;
}

pub trait SaturatingToF32 {
    fn saturating_to_f32(&self) -> f32;
}

pub trait SaturatingToF64 {
    fn saturating_to_f64(&self) -> f64;
}

pub trait Binary {
    fn and(&self, other: &Self) -> Self;
    fn or(&self, other: &Self) -> Self;
    fn xor(&self, other: &Self) -> Self;
    fn not(&self) -> Self;
}

pub trait Signed: Sized {
    fn abs(&self) -> Option<Self>;
    fn signum(&self) -> Self;
    fn is_positive(&self) -> bool;
    fn is_negative(&self) -> bool;
}

pub trait Float {
    fn is_nan(&self) -> bool;
    fn is_infinite(&self) -> bool;
    fn is_finite(&self) -> bool;
    fn is_normal(&self) -> bool;
    fn is_subnormal(&self) -> bool;
    fn floor(&self) -> Self;
    fn ceil(&self) -> Self;
    fn round(&self) -> Self;
    fn trunc(&self) -> Self;
    fn fract(&self) -> Self;
    fn recip(&self) -> Self;
    fn pow(&self, n: &Self) -> Self;
    fn sqrt(&self) -> Self;
    fn exp(&self) -> Self;
    fn exp2(&self) -> Self;
    fn ln(&self) -> Self;
    fn log(&self, base: &Self) -> Self;
    fn log2(&self) -> Self;
    fn log10(&self) -> Self;
    fn cbrt(&self) -> Self;
    fn hypot(&self, n: &Self) -> Self;
    fn sin(&self) -> Self;
    fn cos(&self) -> Self;
    fn tan(&self) -> Self;
    fn asin(&self) -> Self;
    fn acos(&self) -> Self;
    fn atan(&self) -> Self;
    fn atan2(&self, n: &Self) -> Self;
    fn sinh(&self) -> Self;
    fn cosh(&self) -> Self;
    fn tanh(&self) -> Self;
    fn asinh(&self) -> Self;
    fn acosh(&self) -> Self;
    fn atanh(&self) -> Self;
    fn to_degrees(&self) -> Self;
    fn to_radians(&self) -> Self;
}

pub trait PartialEquality {
    fn eq(&self, other: &Self) -> bool;
    fn ne(&self, other: &Self) -> bool;
}

impl<T> PartialEquality for T
where
    T: core::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        core::cmp::PartialEq::eq(self, other)
    }

    fn ne(&self, other: &Self) -> bool {
        core::cmp::PartialEq::ne(self, other)
    }
}

pub trait PartialOrder {
    fn lt(&self, other: &Self) -> bool;
    fn le(&self, other: &Self) -> bool;
    fn gt(&self, other: &Self) -> bool;
    fn ge(&self, other: &Self) -> bool;
}

impl<T> PartialOrder for T
where
    T: core::cmp::PartialOrd,
{
    fn lt(&self, other: &Self) -> bool {
        core::cmp::PartialOrd::lt(self, other)
    }

    fn le(&self, other: &Self) -> bool {
        core::cmp::PartialOrd::le(self, other)
    }

    fn gt(&self, other: &Self) -> bool {
        core::cmp::PartialOrd::gt(self, other)
    }

    fn ge(&self, other: &Self) -> bool {
        core::cmp::PartialOrd::ge(self, other)
    }
}

pub trait Order {
    fn max(&self, other: &Self) -> Self;
    fn min(&self, other: &Self) -> Self;
    fn clamp(&self, min: &Self, max: &Self) -> Self;
}

impl<T> Order for T
where
    T: core::cmp::Ord + Clone,
{
    fn max(&self, other: &Self) -> Self {
        core::cmp::Ord::max(self, other).clone()
    }

    fn min(&self, other: &Self) -> Self {
        core::cmp::Ord::min(self, other).clone()
    }

    fn clamp(&self, min: &Self, max: &Self) -> Self {
        core::cmp::Ord::clamp(self, min, max).clone()
    }
}

pub trait Add {
    fn add(&self, other: &Self) -> Self;
}

pub trait CheckedAdd: Sized {
    fn checked_add(&self, other: &Self) -> Option<Self>;
}

pub trait SaturatingAdd {
    fn saturating_add(&self, other: &Self) -> Self;
}

pub trait WrappingAdd {
    fn wrapping_add(&self, other: &Self) -> Self;
}

pub trait Sub {
    fn sub(&self, other: &Self) -> Self;
}

pub trait CheckedSub: Sized {
    fn checked_sub(&self, other: &Self) -> Option<Self>;
}

pub trait SaturatingSub {
    fn saturating_sub(&self, other: &Self) -> Self;
}

pub trait WrappingSub {
    fn wrapping_sub(&self, other: &Self) -> Self;
}

pub trait Mul {
    fn mul(&self, other: &Self) -> Self;
}

pub trait CheckedMul: Sized {
    fn mul(&self, other: &Self) -> Option<Self>;
}

pub trait SaturatingMul {
    fn saturating_mul(&self, other: &Self) -> Self;
}

pub trait WrappingMul {
    fn wrapping_mul(&self, other: &Self) -> Self;
}

pub trait Div {
    fn div(&self, other: &Self) -> Self;
}

pub trait CheckedDiv: Sized {
    fn checked_div(&self, other: &Self) -> Option<Self>;
}

pub trait Rem {
    fn rem(&self, other: &Self) -> Self;
}

pub trait CheckedRem: Sized {
    fn checked_rem(&self, other: &Self) -> Option<Self>;
}

pub trait Neg {
    fn neg(&self) -> Self;
}

pub trait CheckedNeg: Sized {
    fn checked_neg(&self) -> Option<Self>;
}

pub trait WrappingNeg {
    fn wrapping_neg(&self) -> Self;
}

pub trait Pow {
    fn pow(&self, exp: &u32) -> Self;
}

pub trait CheckedPow: Sized {
    fn checked_pow(&self, exp: &u32) -> Option<Self>;
}

pub trait Euclid {
    fn euclid_div(&self, other: &Self) -> Self;
    fn euclid_rem(&self, other: &Self) -> Self;
}

pub trait CheckedEuclid: Sized {
    fn checked_euclid_div(&self, other: &Self) -> Option<Self>;
    fn checked_euclid_rem(&self, other: &Self) -> Option<Self>;
}

pub trait Hash {
    fn hash(&self, state: &mut dyn core::hash::Hasher);
}

impl<T> Hash for T
where
    T: core::hash::Hash,
{
    fn hash(&self, mut state: &mut dyn core::hash::Hasher) {
        core::hash::Hash::hash(&self, &mut state)
    }
}

pub trait Serialize {
    fn serialize(
        &self,
        serializer: &mut dyn crate::ErasedSerializer,
    ) -> Result<(), crate::ErasedSerdeError>;
}

impl<T> Serialize for T
where
    T: crate::ErasedSerialize,
{
    fn serialize(
        &self,
        serializer: &mut dyn crate::ErasedSerializer,
    ) -> Result<(), crate::ErasedSerdeError> {
        self.erased_serialize(serializer)
    }
}

pub trait Display {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error>;
}

impl<T> Display for T
where
    T: core::fmt::Display,
{
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        core::fmt::Display::fmt(self, f)
    }
}
