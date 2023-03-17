
use melodium::*;
use melodium_doc::Documentation;
use std::path::PathBuf;

pub fn main() {
    let collection = load().unwrap();
    let documentation = Documentation::new(PathBuf::from("/tmp/doc_test"), collection);
    documentation.make_documentation().unwrap();
}
