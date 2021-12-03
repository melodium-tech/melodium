
use std::sync::Arc;
use crate::logic::descriptor::{ModelDescriptor, TreatmentDescriptor};
use crate::logic::collection_pool::CollectionPool;

use super::fs::direct::file_reader::FileReaderModel;
use super::fs::direct::file_writer::FileWriterModel;

use super::fs::direct::read_file::ReadFileTreatment;
use super::fs::direct::write_file::WriteFileTreatment;


pub fn core_collection() -> &'static CollectionPool {

    lazy_static! {
        static ref SINGLETON: CollectionPool = {
            let mut c = CollectionPool::new();

            c.models.insert(&(FileReaderModel::descriptor() as Arc<dyn ModelDescriptor>));
            c.models.insert(&(FileWriterModel::descriptor() as Arc<dyn ModelDescriptor>));

            c.treatments.insert(&(ReadFileTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(WriteFileTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));

            c
        };
    }
    &SINGLETON
}