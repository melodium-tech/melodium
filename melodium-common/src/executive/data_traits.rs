use super::Value;

pub trait DataTrait {
    fn to_i8(&self) -> Value;
    fn to_i16(&self) -> Value;
    fn to_i32(&self) -> Value;
    fn to_i64(&self) -> Value;
    fn to_i128(&self) -> Value;
    fn to_u8(&self) -> Value;
    fn to_u16(&self) -> Value;
    fn to_u32(&self) -> Value;
    fn to_u64(&self) -> Value;
    fn to_u128(&self) -> Value;
    fn to_f32(&self) -> Value;
    fn to_f64(&self) -> Value;
    fn to_bool(&self) -> Value;
    fn to_byte(&self) -> Value;
    fn to_char(&self) -> Value;
    fn to_string(&self) -> Value;

    fn try_to_i8(&self) -> Value;
    fn try_to_i16(&self) -> Value;
    fn try_to_i32(&self) -> Value;
    fn try_to_i64(&self) -> Value;
    fn try_to_i128(&self) -> Value;
    fn try_to_u8(&self) -> Value;
    fn try_to_u16(&self) -> Value;
    fn try_to_u32(&self) -> Value;
    fn try_to_u64(&self) -> Value;
    fn try_to_u128(&self) -> Value;
    fn try_to_f32(&self) -> Value;
    fn try_to_f64(&self) -> Value;
    fn try_to_bool(&self) -> Value;
    fn try_to_byte(&self) -> Value;
    fn try_to_char(&self) -> Value;
    fn try_to_string(&self) -> Value;

    fn saturating_to_i8(&self) -> Value;
    fn saturating_to_i16(&self) -> Value;
    fn saturating_to_i32(&self) -> Value;
    fn saturating_to_i64(&self) -> Value;
    fn saturating_to_i128(&self) -> Value;
    fn saturating_to_u8(&self) -> Value;
    fn saturating_to_u16(&self) -> Value;
    fn saturating_to_u32(&self) -> Value;
    fn saturating_to_u64(&self) -> Value;
    fn saturating_to_u128(&self) -> Value;
    fn saturating_to_f32(&self) -> Value;
    fn saturating_to_f64(&self) -> Value;

    // signed
    fn signed_abs(&self) -> Value;
    fn signed_signum(&self) -> Value;
    fn signed_is_positive(&self) -> Value;
    fn signed_is_negative(&self) -> Value;

    // float
    fn float_is_nan(&self) -> Value;
    fn float_is_infinite(&self) -> Value;
    fn float_is_finite(&self) -> Value;
    fn float_is_normal(&self) -> Value;
    fn float_is_subnormal(&self) -> Value;
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
    fn float_hypot(&self) -> Value;
    fn float_sin(&self) -> Value;
    fn float_cos(&self) -> Value;
    fn float_tan(&self) -> Value;
    fn float_asin(&self) -> Value;
    fn float_acos(&self) -> Value;
    fn float_atan(&self) -> Value;
    fn float_atan2(&self) -> Value;
    fn float_sinh(&self) -> Value;
    fn float_cosh(&self) -> Value;
    fn float_tanh(&self) -> Value;
    fn float_asinh(&self) -> Value;
    fn float_acosh(&self) -> Value;
    fn float_atanh(&self) -> Value;
    fn float_to_degrees(&self) -> Value;
    fn float_to_radians(&self) -> Value;

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

    fn partial_equality_eq(&self, other: &Value) -> Value;
    fn partial_equality_ne(&self, other: &Value) -> Value;

    fn partial_order_lt(&self, other: &Value) -> Value;
    fn partial_order_le(&self, other: &Value) -> Value;
    fn partial_order_gt(&self, other: &Value) -> Value;
    fn partial_order_ge(&self, other: &Value) -> Value;

    fn order_max(&self, other: &Value) -> Value;
    fn order_min(&self, other: &Value) -> Value;
    fn order_clamp(&self, min: &Value, max: &Value) -> Value;

    // Ops traits, one per line
    fn add(&self, other: &Value) -> Value;
    fn checked_add(&self, other: &Value) -> Value;
    fn saturating_add(&self, other: &Value) -> Value;
    fn wrapping_add(&self, other: &Value) -> Value;
    fn sub(&self, other: &Value) -> Value;
    fn checked_sub(&self, other: &Value) -> Value;
    fn saturating_sub(&self, other: &Value) -> Value;
    fn wrapping_sub(&self, other: &Value) -> Value;
    fn mul(&self, other: &Value) -> Value;
    fn checked_mul(&self, other: &Value) -> Value;
    fn saturating_mul(&self, other: &Value) -> Value;
    fn wrapping_mul(&self, other: &Value) -> Value;
    fn div(&self, other: &Value) -> Value;
    fn checked_div(&self, other: &Value) -> Value;
    fn rem(&self, other: &Value) -> Value;
    fn checked_rem(&self, other: &Value) -> Value;
    fn neg(&self) -> Value;
    fn checked_neg(&self) -> Value;
    fn wrapping_neg(&self) -> Value;
    fn pow(&self, exp: &Value) -> Value;
    fn checked_pow(&self, exp: &Value) -> Value;

    // euclid
    fn euclid_div(&self, other: &Value) -> Value;
    fn euclid_rem(&self, other: &Value) -> Value;
    fn checked_euclid_div(&self, other: &Value) -> Value;
    fn checked_euclid_rem(&self, other: &Value) -> Value;
}
