
pub use async_std::channel::Sender;
pub use async_std::channel::Receiver;
pub use async_std::channel::{bounded, unbounded};

#[derive(Debug, Clone)]
pub enum Transmitter {

    I8(Sender<i8>),
    I16(Sender<i16>),
    I32(Sender<i32>),
    I64(Sender<i64>),
    I128(Sender<i128>),

    U8(Sender<u8>),
    U16(Sender<u16>),
    U32(Sender<u32>),
    U64(Sender<u64>),
    U128(Sender<u128>),

    F32(Sender<f32>),
    F64(Sender<f64>),

    Bool(Sender<bool>),
    Byte(Sender<u8>),
    Char(Sender<char>),
    String(Sender<String>),

    VecI8(Sender<Vec<i8>>),
    VecI16(Sender<Vec<i16>>),
    VecI32(Sender<Vec<i32>>),
    VecI64(Sender<Vec<i64>>),
    VecI128(Sender<Vec<i128>>),

    VecU8(Sender<Vec<u8>>),
    VecU16(Sender<Vec<u16>>),
    VecU32(Sender<Vec<u32>>),
    VecU64(Sender<Vec<u64>>),
    VecU128(Sender<Vec<u128>>),

    VecF32(Sender<Vec<f32>>),
    VecF64(Sender<Vec<f64>>),

    VecBool(Sender<Vec<bool>>),
    VecByte(Sender<Vec<u8>>),
    VecChar(Sender<Vec<char>>),
    VecString(Sender<Vec<String>>),

}
