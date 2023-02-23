
use melodium_macro::{mel_function, mel_treatment, mel_model, mel_context};
use melodium_core::*;

/* 
///
/// Documentatioooon
/// 
#[mel_function]
pub fn truc(hey: Vec<u8>) -> i32 {
    let machin = "bidule";
    let chouette = 1 + 54;
    return chouette
}

/// 
/// Ça fait quelquechose
/// 
#[mel_treatment(
    default un 72
    model ukulele Ukulele 
    input haut Stream<u64>
    input bas Stream<string>
    output droite Block<void>
    output gauche Stream<u64>
)]
pub async fn treatment_chose(un: u64, deux: string, trois: Vec<void>) {

    while let Ok(num) = haut.recv_u64().await {
        let result = num.into_iter().map(|i| i + un).collect();
        gauche.send_multiple_u64(result).await;
    }
}*/

/* 
/// Do some stuff
#[mel_model(
    param bidule Vec<void> none
    source stuff (truc muche) (gauche Stream<f32> droite Stream<Vec<f64>>)
    initialize loading
    shutdown ending
)]
#[derive(Debug)]
pub struct MyModel {
    address: u64,
}

impl MyModel {
    pub fn new(model: std::sync::Weak<MyModelModel>) -> Self {
        Self {
            address: 0
        }
    }

    fn loading(&self) {

    }

    fn ending(&self) {

    }
}*/

#[mel_context]
pub struct Coucou {
    message: string,
    nombre: i128,
    rien: void,
}
