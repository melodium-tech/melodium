
#[derive(Clone, PartialEq, Debug)]
pub enum Value {

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    F32(f32),
    F64(f64),

    Bool(bool),
    Byte(u8),
    Char(char),
    String(String),

    VecI8(Vec<i8>),
    VecI16(Vec<i16>),
    VecI32(Vec<i32>),
    VecI64(Vec<i64>),
    VecI128(Vec<i128>),

    VecU8(Vec<u8>),
    VecU16(Vec<u16>),
    VecU32(Vec<u32>),
    VecU64(Vec<u64>),
    VecU128(Vec<u128>),

    VecF32(Vec<f32>),
    VecF64(Vec<f64>),

    VecBool(Vec<bool>),
    VecByte(Vec<u8>),
    VecChar(Vec<char>),
    VecString(Vec<String>),

}
