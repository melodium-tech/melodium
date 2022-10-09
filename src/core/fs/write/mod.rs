
use crate::core::prelude::*;
pub mod files_writer;

pub fn register(mut c: &mut CollectionPool) {

    files_writer::model_host::register(&mut c);
    files_writer::write_file_treatment::register(&mut c);
}
