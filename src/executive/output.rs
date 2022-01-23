
use std::sync::Arc;
use super::transmitter::*;

#[derive(Debug, Clone)]
pub enum Output {
    I8(Arc<SendTransmitter<i8>>),
    I16(Arc<SendTransmitter<i16>>),
    I32(Arc<SendTransmitter<i32>>),
    I64(Arc<SendTransmitter<i64>>),
    I128(Arc<SendTransmitter<i128>>),

    U8(Arc<SendTransmitter<u8>>),
    U16(Arc<SendTransmitter<u16>>),
    U32(Arc<SendTransmitter<u32>>),
    U64(Arc<SendTransmitter<u64>>),
    U128(Arc<SendTransmitter<u128>>),

    F32(Arc<SendTransmitter<f32>>),
    F64(Arc<SendTransmitter<f64>>),

    Bool(Arc<SendTransmitter<bool>>),
    Byte(Arc<SendTransmitter<u8>>),
    Char(Arc<SendTransmitter<char>>),
    String(Arc<SendTransmitter<String>>),

    VecI8(Arc<SendTransmitter<Vec<i8>>>),
    VecI16(Arc<SendTransmitter<Vec<i16>>>),
    VecI32(Arc<SendTransmitter<Vec<i32>>>),
    VecI64(Arc<SendTransmitter<Vec<i64>>>),
    VecI128(Arc<SendTransmitter<Vec<i128>>>),

    VecU8(Arc<SendTransmitter<Vec<u8>>>),
    VecU16(Arc<SendTransmitter<Vec<u16>>>),
    VecU32(Arc<SendTransmitter<Vec<u32>>>),
    VecU64(Arc<SendTransmitter<Vec<u64>>>),
    VecU128(Arc<SendTransmitter<Vec<u128>>>),

    VecF32(Arc<SendTransmitter<Vec<f32>>>),
    VecF64(Arc<SendTransmitter<Vec<f64>>>),

    VecBool(Arc<SendTransmitter<Vec<bool>>>),
    VecByte(Arc<SendTransmitter<Vec<u8>>>),
    VecChar(Arc<SendTransmitter<Vec<char>>>),
    VecString(Arc<SendTransmitter<Vec<String>>>),
}
