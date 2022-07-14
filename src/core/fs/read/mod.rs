
use crate::core::prelude::*;
pub mod direct_reading;
pub mod files_reader;
pub mod read_file;
pub mod reading;
pub mod unaccessible;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(files_reader::FilesReaderModel::descriptor() as Arc<dyn ModelDescriptor>));

    direct_reading::direct_reading_treatment::register(&mut c);
    read_file::read_file_treatment::register(&mut c);
    reading::reading_treatment::register(&mut c);
    unaccessible::unaccessible_treatment::register(&mut c);
}
