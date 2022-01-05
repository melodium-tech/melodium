
use std::sync::Arc;
use crate::logic::descriptor::{ModelDescriptor, TreatmentDescriptor};
use crate::logic::collection_pool::CollectionPool;

use super::fs::direct::file_reader::FileReaderModel;
use super::fs::direct::file_writer::FileWriterModel;

use super::fs::direct::read_file::ReadFileTreatment;
use super::fs::direct::write_file::WriteFileTreatment;

use super::generation::scalar_u8_generator::ScalarU8Generator;

use super::generation::generate_scalar_u8::GenerateScalarU8;

use super::cast::without_loss::*;

use super::arithmetic::add_scalar::*;

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

            // Cast for f32
            c.treatments.insert(&(CastScalarF32ToF64::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u8
            c.treatments.insert(&(CastScalarU8ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU8ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU8ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU8ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU8ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU8ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU8ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU8ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u16
            c.treatments.insert(&(CastScalarU16ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU16ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU16ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU16ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU16ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU16ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u32
            c.treatments.insert(&(CastScalarU32ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU32ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU32ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU32ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u64
            c.treatments.insert(&(CastScalarU64ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU64ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u128

            // Casts for i8
            c.treatments.insert(&(CastScalarI8ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI8ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI8ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI8ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for i16
            c.treatments.insert(&(CastScalarI16ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI16ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI16ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for i32
            c.treatments.insert(&(CastScalarI32ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI32ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for i64
            c.treatments.insert(&(CastScalarI64ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));


            // Scalar additions
            c.treatments.insert(&(AddScalarI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            c.treatments.insert(&(AddScalarU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarU128::descriptor() as Arc<dyn TreatmentDescriptor>));

            c.treatments.insert(&(AddScalarF32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(AddScalarF64::descriptor() as Arc<dyn TreatmentDescriptor>));

            c.treatments.insert(&(U8ToByte::descriptor() as Arc<dyn TreatmentDescriptor>));

            c.treatments.insert(&(ReadTcpConnectionTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(WriteTcpConnectionTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));

            c
        };
    }
    &SINGLETON
}