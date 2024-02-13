use melodium_macro::mel_function;

pub mod char;
pub mod float;

/// Return the smallest value that can be represented by the type.
#[mel_function(
    generic B (Bounded)
)]
pub fn min() -> B {
    let b = generics.get("B").unwrap();

    b.bounded_min()
}

/// Return the highest value that can be represented by the type.
#[mel_function(
    generic B (Bounded)
)]
pub fn max() -> B {
    let b = generics.get("B").unwrap();

    b.bounded_max()
}
