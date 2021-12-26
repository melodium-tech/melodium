
use std::sync::Arc;
use crate::logic::descriptor::{ModelDescriptor, TreatmentDescriptor};
use crate::logic::collection_pool::CollectionPool;

use super::fs::direct::file_reader::FileReaderModel;
use super::fs::direct::file_writer::FileWriterModel;

use super::fs::direct::read_file::ReadFileTreatment;
use super::fs::direct::write_file::WriteFileTreatment;

use super::generation::scalar_u8_generator::ScalarU8Generator;

use super::generation::generate_scalar_u8::GenerateScalarU8;

use super::arithmetic::add_scalar_u8::AddScalarU8;

use super::conversion::u8_to_byte::U8ToByte;

use super::net::tcp_listener::TcpListenerModel;
use super::net::read_tcp_connection::ReadTcpConnectionTreatment;
use super::net::write_tcp_connection::WriteTcpConnectionTreatment;


pub fn core_collection() -> &'static CollectionPool {

    lazy_static! {
        static ref SINGLETON: CollectionPool = {
            let mut c = CollectionPool::new();

            c.models.insert(&(FileReaderModel::descriptor() as Arc<dyn ModelDescriptor>));
            c.models.insert(&(FileWriterModel::descriptor() as Arc<dyn ModelDescriptor>));

            c.models.insert(&(ScalarU8Generator::descriptor() as Arc<dyn ModelDescriptor>));

            c.models.insert(&(TcpListenerModel::descriptor() as Arc<dyn ModelDescriptor>));

            c.treatments.insert(&(ReadFileTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(WriteFileTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));

            c.treatments.insert(&(GenerateScalarU8::descriptor() as Arc<dyn TreatmentDescriptor>));

            c.treatments.insert(&(AddScalarU8::descriptor() as Arc<dyn TreatmentDescriptor>));

            c.treatments.insert(&(U8ToByte::descriptor() as Arc<dyn TreatmentDescriptor>));

            c.treatments.insert(&(ReadTcpConnectionTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(WriteTcpConnectionTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));

            c
        };
    }
    &SINGLETON
}