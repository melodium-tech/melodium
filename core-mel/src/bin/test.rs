
pub fn main() {

    let collection = core_mel::__mel_package::MelPackage::new().collection();
    for id in collection.identifiers() {
        println!("{id} :");
        println!("{:#?}", collection.get(&id).unwrap());
    }
}