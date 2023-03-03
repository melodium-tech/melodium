

fn main() {
    let doc = melodium_doc::Documentation::new(std::path::PathBuf::from("/tmp/doc_test"), core_mel::__mel_package::MelPackage::new().collection());

    doc.make_documentation().unwrap();
}

