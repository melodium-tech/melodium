
use crate::core::prelude::*;
pub mod file_reader;
pub mod file_writer;
pub mod read_file;
pub mod write_file;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(file_reader::FileReaderModel::descriptor() as Arc<dyn ModelDescriptor>));
    c.models.insert(&(file_writer::FileWriterModel::descriptor() as Arc<dyn ModelDescriptor>));

    read_file::file_reader_treatment::register(&mut c);
    write_file::file_writer_treatment::register(&mut c);
}

