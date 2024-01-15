pub trait DataTrait: Sized {
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

    // signed
    fn signed_abs(&self) -> Option<Self>;
    fn signed_signum(&self) -> Self;
    fn signed_is_positive(&self) -> bool;
    fn signed_is_negative(&self) -> bool;

    // float
    fn float_is_nan(&self) -> bool;
    fn float_is_infinite(&self) -> bool;
    fn float_is_finite(&self) -> bool;
    fn float_is_normal(&self) -> bool;
    fn float_is_subnormal(&self) -> bool;
    fn float_floor(&self) -> Self;
    fn float_ceil(&self) -> Self;
    fn float_round(&self) -> Self;
    fn float_trunc(&self) -> Self;
    fn float_fract(&self) -> Self;
    fn float_recip(&self) -> Self;
    fn float_pow(&self, n: &Self) -> Self;
    fn float_sqrt(&self) -> Self;
    fn float_exp(&self) -> Self;
    fn float_exp2(&self) -> Self;
    fn float_ln(&self) -> Self;
    fn float_log(&self, base: &Self) -> Self;
    fn float_log2(&self) -> Self;
    fn float_log10(&self) -> Self;
    fn float_cbrt(&self) -> Self;
    fn float_hypot(&self, n: &Self) -> Self;
    fn float_sin(&self) -> Self;
    fn float_cos(&self) -> Self;
    fn float_tan(&self) -> Self;
    fn float_asin(&self) -> Self;
    fn float_acos(&self) -> Self;
    fn float_atan(&self) -> Self;
    fn float_atan2(&self, n: &Self) -> Self;
    fn float_sinh(&self) -> Self;
    fn float_cosh(&self) -> Self;
    fn float_tanh(&self) -> Self;
    fn float_asinh(&self) -> Self;
    fn float_acosh(&self) -> Self;
    fn float_atanh(&self) -> Self;
    fn float_to_degrees(&self) -> Self;
    fn float_to_radians(&self) -> Self;

    // text (may not be a trait as everything is only for 'string' type)
    /*fn text_contains(&self, other: &Value) -> Value;
    fn text_ends_with(&self, other: &Value) -> Value;
    fn text_is_ascii(&self) -> Value;
    fn text_len(&self) -> Value;
    fn text_replace(&self, from: &Value, to: &Value) -> Value;
    fn text_to_lowercase(&self) -> Value;
    fn text_to_uppercase(&self) -> Value;
    fn text_trim(&self) -> Value;
    fn text_trim_start(&self) -> Value;
    fn text_trim_start_matches(&self, trim: &Value) -> Value;
    fn text_trim_end(&self) -> Value;
    fn text_trim_end_matches(&self, trim: &Value) -> Value;*/

    fn partial_equality_eq(&self, other: &Self) -> bool;
    fn partial_equality_ne(&self, other: &Self) -> bool;

    fn partial_order_lt(&self, other: &Self) -> bool;
    fn partial_order_le(&self, other: &Self) -> bool;
    fn partial_order_gt(&self, other: &Self) -> bool;
    fn partial_order_ge(&self, other: &Self) -> bool;

    fn order_max(&self, other: &Self) -> Self;
    fn order_min(&self, other: &Self) -> Self;
    fn order_clamp(&self, min: &Self, max: &Self) -> Self;

    // Ops traits, one per line
    fn add(&self, other: &Self) -> Self;
    fn checked_add(&self, other: &Self) -> Option<Self>;
    fn saturating_add(&self, other: &Self) -> Self;
    fn wrapping_add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn checked_sub(&self, other: &Self) -> Option<Self>;
    fn saturating_sub(&self, other: &Self) -> Self;
    fn wrapping_sub(&self, other: &Self) -> Self;
    fn mul(&self, other: &Self) -> Self;
    fn checked_mul(&self, other: &Self) -> Option<Self>;
    fn saturating_mul(&self, other: &Self) -> Self;
    fn wrapping_mul(&self, other: &Self) -> Self;
    fn div(&self, other: &Self) -> Self;
    fn checked_div(&self, other: &Self) -> Option<Self>;
    fn rem(&self, other: &Self) -> Self;
    fn checked_rem(&self, other: &Self) -> Option<Self>;
    fn neg(&self) -> Self;
    fn checked_neg(&self) -> Option<Self>;
    fn wrapping_neg(&self) -> Self;
    fn pow(&self, exp: &u32) -> Self;
    fn checked_pow(&self, exp: &u32) -> Option<Self>;

    // euclid
    fn euclid_div(&self, other: &Self) -> Self;
    fn euclid_rem(&self, other: &Self) -> Self;
    fn checked_euclid_div(&self, other: &Self) -> Option<Self>;
    fn checked_euclid_rem(&self, other: &Self) -> Option<Self>;
}
