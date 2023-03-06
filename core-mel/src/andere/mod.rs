use melodium_core::*;
use melodium_macro::{mel_context, mel_function, mel_model, mel_treatment};

pub mod lol;

#[mel_context]
pub struct Andere {
    message: string,
    nombre: i128,
    rien: void,
}
