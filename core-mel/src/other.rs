
use melodium_macro::{mel_function, mel_treatment};
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
}
