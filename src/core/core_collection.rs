
use std::sync::Arc;
use crate::logic::descriptor::{ModelDescriptor, TreatmentDescriptor};
use crate::logic::collection_pool::CollectionPool;

use super::fs::direct::file_reader::FileReaderModel;
use super::fs::direct::file_writer::FileWriterModel;

use super::fs::direct::read_file::ReadFileTreatment;
use super::fs::direct::write_file::WriteFileTreatment;

use super::generation::scalar_u8_generator::ScalarU8Generator;

use super::generation::generate_scalar_u8::GenerateScalarU8;

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

            super::cast::lossless_scalar::register(&mut c);
            super::cast::lossless_vector::register(&mut c);
            super::cast::lossy_scalar::register(&mut c);
            super::cast::lossy_vector::register(&mut c);

            super::conversion::scalar_to_string::register(&mut c);
            super::conversion::vector_to_string::register(&mut c);
            super::conversion::scalar_float_to_integer::register(&mut c);
            super::conversion::vector_float_to_integer::register(&mut c);

            super::arithmetic::add_scalar::register(&mut c);

            c.treatments.insert(&(ReadTcpConnectionTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(WriteTcpConnectionTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));

            c
        };
    }
    &SINGLETON
}

