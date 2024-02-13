use super::Value;
use core::{fmt::Formatter, hash::Hasher};
use erased_serde::{serialize_trait_object, Serialize, Serializer};

pub trait DataTrait: Serialize {
    fn to_i8(&self) -> i8;
    fn to_i16(&self) -> i16;
    fn to_i32(&self) -> i32;
    fn to_i64(&self) -> i64;
    fn to_i128(&self) -> i128;
    fn to_u8(&self) -> u8;
    fn to_u16(&self) -> u16;
    fn to_u32(&self) -> u32;
    fn to_u64(&self) -> u64;
    fn to_u128(&self) -> u128;
    fn to_f32(&self) -> f32;
    fn to_f64(&self) -> f64;
    fn to_bool(&self) -> bool;
    fn to_byte(&self) -> u8;
    fn to_char(&self) -> char;
    fn to_string(&self) -> String;

    fn try_to_i8(&self) -> Option<i8>;
    fn try_to_i16(&self) -> Option<i16>;
    fn try_to_i32(&self) -> Option<i32>;
    fn try_to_i64(&self) -> Option<i64>;
    fn try_to_i128(&self) -> Option<i128>;
    fn try_to_u8(&self) -> Option<u8>;
    fn try_to_u16(&self) -> Option<u16>;
    fn try_to_u32(&self) -> Option<u32>;
    fn try_to_u64(&self) -> Option<u64>;
    fn try_to_u128(&self) -> Option<u128>;
    fn try_to_f32(&self) -> Option<f32>;
    fn try_to_f64(&self) -> Option<f64>;
    fn try_to_bool(&self) -> Option<bool>;
    fn try_to_byte(&self) -> Option<u8>;
    fn try_to_char(&self) -> Option<char>;
    fn try_to_string(&self) -> Option<String>;

    fn saturating_to_i8(&self) -> i8;
    fn saturating_to_i16(&self) -> i16;
    fn saturating_to_i32(&self) -> i32;
    fn saturating_to_i64(&self) -> i64;
    fn saturating_to_i128(&self) -> i128;
    fn saturating_to_u8(&self) -> u8;
    fn saturating_to_u16(&self) -> u16;
    fn saturating_to_u32(&self) -> u32;
    fn saturating_to_u64(&self) -> u64;
    fn saturating_to_u128(&self) -> u128;
    fn saturating_to_f32(&self) -> f32;
    fn saturating_to_f64(&self) -> f64;

    // binary
    fn binary_and(&self, other: &Value) -> Value;
    fn binary_or(&self, other: &Value) -> Value;
    fn binary_xor(&self, other: &Value) -> Value;
    fn binary_not(&self) -> Value;

    // signed
    fn signed_abs(&self) -> Option<Value>;
    fn signed_signum(&self) -> Value;
    fn signed_is_positive(&self) -> bool;
    fn signed_is_negative(&self) -> bool;

    // float
    fn float_is_nan(&self) -> bool;
    fn float_is_infinite(&self) -> bool;
    fn float_is_finite(&self) -> bool;
    fn float_is_normal(&self) -> bool;
    fn float_is_subnormal(&self) -> bool;
    fn float_floor(&self) -> Value;
    fn float_ceil(&self) -> Value;
    fn float_round(&self) -> Value;
    fn float_trunc(&self) -> Value;
    fn float_fract(&self) -> Value;
    fn float_recip(&self) -> Value;
    fn float_pow(&self, n: &Value) -> Value;
    fn float_sqrt(&self) -> Value;
    fn float_exp(&self) -> Value;
    fn float_exp2(&self) -> Value;
    fn float_ln(&self) -> Value;
    fn float_log(&self, base: &Value) -> Value;
    fn float_log2(&self) -> Value;
    fn float_log10(&self) -> Value;
    fn float_cbrt(&self) -> Value;
    fn float_hypot(&self, n: &Value) -> Value;
    fn float_sin(&self) -> Value;
    fn float_cos(&self) -> Value;
    fn float_tan(&self) -> Value;
    fn float_asin(&self) -> Value;
    fn float_acos(&self) -> Value;
    fn float_atan(&self) -> Value;
    fn float_atan2(&self, n: &Value) -> Value;
    fn float_sinh(&self) -> Value;
    fn float_cosh(&self) -> Value;
    fn float_tanh(&self) -> Value;
    fn float_asinh(&self) -> Value;
    fn float_acosh(&self) -> Value;
    fn float_atanh(&self) -> Value;
    fn float_to_degrees(&self) -> Value;
    fn float_to_radians(&self) -> Value;

    fn partial_equality_eq(&self, other: &Value) -> bool;
    fn partial_equality_ne(&self, other: &Value) -> bool;

    fn partial_order_lt(&self, other: &Value) -> bool;
    fn partial_order_le(&self, other: &Value) -> bool;
    fn partial_order_gt(&self, other: &Value) -> bool;
    fn partial_order_ge(&self, other: &Value) -> bool;

    fn order_max(&self, other: &Value) -> Value;
    fn order_min(&self, other: &Value) -> Value;
    fn order_clamp(&self, min: &Value, max: &Value) -> Value;

    // Ops traits, one per line
    fn add(&self, other: &Value) -> Value;
    fn checked_add(&self, other: &Value) -> Option<Value>;
    fn saturating_add(&self, other: &Value) -> Value;
    fn wrapping_add(&self, other: &Value) -> Value;
    fn sub(&self, other: &Value) -> Value;
    fn checked_sub(&self, other: &Value) -> Option<Value>;
    fn saturating_sub(&self, other: &Value) -> Value;
    fn wrapping_sub(&self, other: &Value) -> Value;
    fn mul(&self, other: &Value) -> Value;
    fn checked_mul(&self, other: &Value) -> Option<Value>;
    fn saturating_mul(&self, other: &Value) -> Value;
    fn wrapping_mul(&self, other: &Value) -> Value;
    fn div(&self, other: &Value) -> Value;
    fn checked_div(&self, other: &Value) -> Option<Value>;
    fn rem(&self, other: &Value) -> Value;
    fn checked_rem(&self, other: &Value) -> Option<Value>;
    fn neg(&self) -> Value;
    fn checked_neg(&self) -> Option<Value>;
    fn wrapping_neg(&self) -> Value;
    fn pow(&self, exp: &u32) -> Value;
    fn checked_pow(&self, exp: &u32) -> Option<Value>;

    // euclid
    fn euclid_div(&self, other: &Value) -> Value;
    fn euclid_rem(&self, other: &Value) -> Value;
    fn checked_euclid_div(&self, other: &Value) -> Option<Value>;
    fn checked_euclid_rem(&self, other: &Value) -> Option<Value>;

    fn hash(&self, state: &mut dyn Hasher);

    fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), erased_serde::Error>;

    fn display(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error>;
}

serialize_trait_object!(DataTrait);
