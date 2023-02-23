
use melodium_macro::{mel_function, mel_treatment, mel_model, mel_context};
use melodium_core::*;

#[mel_context]
pub struct Coucou {
    message: string,
    nombre: i128,
    rien: void,
}
