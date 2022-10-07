
use crate::core::prelude::*;
pub mod files_writer;
pub mod write_file;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(files_writer::FilesWriterModel::descriptor() as std::sync::Arc<dyn ModelDescriptor>));

    write_file::write_file_treatment::register(&mut c);
}
