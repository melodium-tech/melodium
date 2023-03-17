
use crate::core::prelude::*;
pub mod direct_reading;
pub mod files_reader;

pub fn register(mut c: &mut CollectionPool) {

    direct_reading::direct_reading_treatment::register(&mut c);
    files_reader::model_host::register(&mut c);
    files_reader::reading_source::register(&mut c);
    files_reader::unaccessible_source::register(&mut c);
    files_reader::read_file_treatment::register(&mut c);
}
