use melodium_core::*;
use melodium_macro::{mel_context, mel_function, mel_model, mel_treatment};

#[mel_context]
pub struct Coucou {
    message: string,
    nombre: i128,
    rien: void,
}
