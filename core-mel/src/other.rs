
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
    default un -78
    model ukulele Ukulele 
    input haut Stream<u64>
    input bas Stream<string>
    output droite Block<void>
    output gauche Stream<Vec<bool>>
)]
pub fn treatment_chose(un: i64, deux: string, trois: Vec<void>) {

}
