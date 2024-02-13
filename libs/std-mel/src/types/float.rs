use melodium_macro::mel_function;

/// Return the positive infinity for floating type.
#[mel_function(
    generic F (Float)
)]
pub fn infinity() -> F {
    let f = generics.get("F").unwrap();

    f.float_infinity()
}

/// Return the negative infinity for floating type.
#[mel_function(
    generic F (Float)
)]
pub fn neg_infinity() -> F {
    let f = generics.get("F").unwrap();

    f.float_neg_infinity()
}

/// Return the not-a-number value for floating type.
#[mel_function(
    generic F (Float)
)]
pub fn nan() -> F {
    let f = generics.get("F").unwrap();

    f.float_nan()
}
