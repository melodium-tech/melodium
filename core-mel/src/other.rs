
use melodium_macro::{mel_function, mel_treatment, mel_model, mel_context};
use melodium_core::*;


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
    model ukulele { crate::other::MyModel }
    input haut Stream<u64>
    input bas Stream<string>
    output droite Block<void>
    output gauche Stream<u64>
)]
pub async fn treatment_chose(un: u64, deux: string, trois: Vec<void>) {

    while let Ok(num) = haut.recv_u64().await {
        let result = num.into_iter().map(|i| i + un).collect();
        gauche.send_u64(result).await;
    }
}

/// Do some stuff
#[mel_model(
    param bidule Vec<void> none
    source stuff (Coucou) (gauche Stream<f32> droite Stream<Vec<f64>>)
    initialize loading
    continuous (trucs run1 run2)
    shutdown ending
)]
#[derive(Debug)]
pub struct MyModel {
    model: std::sync::Weak<MyModelModel>,
    address: u64,
}

impl MyModel {
    pub fn new(model: std::sync::Weak<MyModelModel>) -> Self {
        Self {
            model,
            address: 0
        }
    }

    fn loading(&self) {

    }

    fn ending(&self) {

    }

    pub async fn run1(&self) {
        
    }

    pub async fn run2(&self) {

    }

    pub async fn trucs(&self) {
        let model = self.model.upgrade().unwrap();
        let passed_model = std::sync::Arc::clone(&model);

        model.new_stuff(
            None,
            Coucou {
                message: "Coucou".to_string(),
                nombre: 24,
                rien: (),
            },
            Some(Box::new( move |outputs| {

                let id = passed_model.parameter("truc");
                vec![]
            }))
        ).await;

    }
}

#[mel_context]
pub struct Coucou {
    message: string,
    nombre: i128,
    rien: void,
}