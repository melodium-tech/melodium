
use melodium_macro::{mel_package, mel_function};
use melodium_core::*;
pub mod other;
mod andere;

#[mel_function()]
pub fn truc(hey: void) -> i32 {
    3
}

mel_package!();
