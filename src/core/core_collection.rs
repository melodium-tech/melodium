
use std::sync::Arc;
use crate::logic::descriptor::{ModelDescriptor, TreatmentDescriptor};
use crate::logic::collection_pool::CollectionPool;

use super::fs::direct::file_reader::FileReaderModel;
use super::fs::direct::file_writer::FileWriterModel;

use super::fs::direct::read_file::ReadFileTreatment;
use super::fs::direct::write_file::WriteFileTreatment;

use super::generation::scalar_u8_generator::ScalarU8Generator;

use super::generation::generate_scalar_u8::GenerateScalarU8;

use super::cast::lossless_scalar::*;
use super::cast::lossless_vector::*;
use super::cast::lossy_scalar::*;
use super::cast::lossy_vector::*;

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

            // Casts for f32 and f64
            c.treatments.insert(&(CastScalarF32ToF64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarF64ToF32::descriptor() as Arc<dyn TreatmentDescriptor>));

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

            // Casts for f32 and f64
            c.treatments.insert(&(CastVectorF32ToF64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorF64ToF32::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u8
            c.treatments.insert(&(CastVectorU8ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU8ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU8ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU8ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU8ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU8ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU8ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU8ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u16
            c.treatments.insert(&(CastVectorU16ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU16ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU16ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU16ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU16ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU16ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u32
            c.treatments.insert(&(CastVectorU32ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU32ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU32ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU32ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u64
            c.treatments.insert(&(CastVectorU64ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU64ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for u128

            // Casts for i8
            c.treatments.insert(&(CastVectorI8ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI8ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI8ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI8ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for i16
            c.treatments.insert(&(CastVectorI16ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI16ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI16ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for i32
            c.treatments.insert(&(CastVectorI32ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI32ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for i64
            c.treatments.insert(&(CastVectorI64ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Casts for i128

            // Lossy casts for u8
            c.treatments.insert(&(CastScalarU8ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u16
            c.treatments.insert(&(CastScalarU16ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU16ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU16ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u32
            c.treatments.insert(&(CastScalarU32ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU32ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU32ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU32ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU32ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u64
            c.treatments.insert(&(CastScalarU64ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU64ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU64ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU64ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU64ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU64ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU64ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u128
            c.treatments.insert(&(CastScalarU128ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU128ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU128ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU128ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU128ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU128ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU128ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU128ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarU128ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i8
            c.treatments.insert(&(CastScalarI8ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI8ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI8ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI8ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI8ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i16
            c.treatments.insert(&(CastScalarI16ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI16ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI16ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI16ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI16ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI16ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i32
            c.treatments.insert(&(CastScalarI32ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI32ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI32ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI32ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI32ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI32ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI32ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i64
            c.treatments.insert(&(CastScalarI64ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI64ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI64ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI64ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI64ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI64ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI64ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI64ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i128
            c.treatments.insert(&(CastScalarI128ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI128ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI128ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI128ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI128ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI128ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI128ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI128ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastScalarI128ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u8
            c.treatments.insert(&(CastVectorU8ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u16
            c.treatments.insert(&(CastVectorU16ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU16ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU16ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u32
            c.treatments.insert(&(CastVectorU32ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU32ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU32ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU32ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU32ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u64
            c.treatments.insert(&(CastVectorU64ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU64ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU64ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU64ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU64ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU64ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU64ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for u128
            c.treatments.insert(&(CastVectorU128ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU128ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU128ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU128ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU128ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU128ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU128ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU128ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorU128ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i8
            c.treatments.insert(&(CastVectorI8ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI8ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI8ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI8ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI8ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i16
            c.treatments.insert(&(CastVectorI16ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI16ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI16ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI16ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI16ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI16ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i32
            c.treatments.insert(&(CastVectorI32ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI32ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI32ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI32ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI32ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI32ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI32ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i64
            c.treatments.insert(&(CastVectorI64ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI64ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI64ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI64ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI64ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI64ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI64ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI64ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));

            // Lossy casts for i128
            c.treatments.insert(&(CastVectorI128ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI128ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI128ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI128ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI128ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI128ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI128ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI128ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(CastVectorI128ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));


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