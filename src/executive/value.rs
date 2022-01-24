
use std::fmt::*;

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

impl Display for Value {
    
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        //write!(f, "*To implement*")

        match self {
            Value::I8(v) => write!(f, "{}", v),
            Value::I16(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::I128(v) => write!(f, "{}", v),
            Value::U8(v) => write!(f, "{}", v),
            Value::U16(v) => write!(f, "{}", v),
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),
            Value::U128(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Byte(v) => write!(f, "{}", v),
            Value::Char(v) => write!(f, "'{}'", v),
            Value::String(v) => write!(f, "\"{}\"", v),

            Value::VecI8(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecI16(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecI32(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecI64(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecI128(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecU8(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecU16(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecU32(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecU64(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecU128(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecF32(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecF64(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecBool(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecByte(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecChar(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("'{}'", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
            Value::VecString(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("\"{}\"", v)).collect();
                write!(f, "[{}]", list.join(", "))
            },
        }
    }
}
