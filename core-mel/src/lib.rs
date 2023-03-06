use melodium_core::*;
use melodium_macro::{mel_function, mel_package};
mod andere;
pub mod ops;
pub mod other;

#[mel_function()]
pub fn truc(hey: void) -> i32 {
    3
}

mel_package!();
