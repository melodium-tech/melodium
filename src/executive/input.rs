
use std::sync::Arc;
use super::transmitter::*;

#[derive(Debug, Clone)]
pub enum Input {
    I8(Arc<RecvTransmitter<i8>>),
    I16(Arc<RecvTransmitter<i16>>),
    I32(Arc<RecvTransmitter<i32>>),
    I64(Arc<RecvTransmitter<i64>>),
    I128(Arc<RecvTransmitter<i128>>),

    U8(Arc<RecvTransmitter<u8>>),
    U16(Arc<RecvTransmitter<u16>>),
    U32(Arc<RecvTransmitter<u32>>),
    U64(Arc<RecvTransmitter<u64>>),
    U128(Arc<RecvTransmitter<u128>>),

    F32(Arc<RecvTransmitter<f32>>),
    F64(Arc<RecvTransmitter<f64>>),

    Bool(Arc<RecvTransmitter<bool>>),
    Byte(Arc<RecvTransmitter<u8>>),
    Char(Arc<RecvTransmitter<char>>),
    String(Arc<RecvTransmitter<String>>),

    VecI8(Arc<RecvTransmitter<Vec<i8>>>),
    VecI16(Arc<RecvTransmitter<Vec<i16>>>),
    VecI32(Arc<RecvTransmitter<Vec<i32>>>),
    VecI64(Arc<RecvTransmitter<Vec<i64>>>),
    VecI128(Arc<RecvTransmitter<Vec<i128>>>),

    VecU8(Arc<RecvTransmitter<Vec<u8>>>),
    VecU16(Arc<RecvTransmitter<Vec<u16>>>),
    VecU32(Arc<RecvTransmitter<Vec<u32>>>),
    VecU64(Arc<RecvTransmitter<Vec<u64>>>),
    VecU128(Arc<RecvTransmitter<Vec<u128>>>),

    VecF32(Arc<RecvTransmitter<Vec<f32>>>),
    VecF64(Arc<RecvTransmitter<Vec<f64>>>),

    VecBool(Arc<RecvTransmitter<Vec<bool>>>),
    VecByte(Arc<RecvTransmitter<Vec<u8>>>),
    VecChar(Arc<RecvTransmitter<Vec<char>>>),
    VecString(Arc<RecvTransmitter<Vec<String>>>),
}
