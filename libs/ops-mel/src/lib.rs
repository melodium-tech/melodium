#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_macro::{mel_function, mel_package};

pub mod bool;
pub mod byte;
pub mod char;
pub mod f32;
pub mod f64;
pub mod i128;
pub mod i16;
pub mod i32;
pub mod i64;
pub mod i8;
pub mod string;
pub mod u128;
pub mod u16;
pub mod u32;
pub mod u64;
pub mod u8;

/// Return whether `a` is equal to `b`
#[mel_function(
    generic T
)]
pub fn equal(a: T, b: T) -> bool {
    a == b
}

/// Compares and gives the maximum of two values.
#[mel_function(
    generic T
)]
pub fn max(a: T, b: T) -> T {
    a
}

mel_package!();
