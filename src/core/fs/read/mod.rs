
use crate::core::prelude::*;
pub mod direct_reading;
pub mod files_reader;
pub mod read_file;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(files_reader::FilesReaderModel::descriptor() as Arc<dyn ModelDescriptor>));
    files_reader::reading_source::register(&mut c);
    files_reader::unaccessible_source::register(&mut c);

    direct_reading::direct_reading_treatment::register(&mut c);
    read_file::read_file_treatment::register(&mut c);
}
