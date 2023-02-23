
use melodium_macro::{mel_function, mel_treatment, mel_model, mel_context};
use melodium_core::*;

pub mod lol;

#[mel_context]
pub struct Andere {
    message: string,
    nombre: i128,
    rien: void,
}
