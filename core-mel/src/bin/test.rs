
pub fn main() {

    let collection = core_mel::__mel_collection();
    for id in collection.identifiers() {
        println!("{id} :");
        println!("{:#?}", collection.get(&id).unwrap());
    }
}