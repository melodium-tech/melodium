
pub fn main() {

    for id in core_mel::__mel_collection().identifiers() {
        println!("{id}")
    }
}