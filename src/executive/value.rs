
use std::fmt::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Value {

    Void(()),

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

    VecVoid(Vec<()>),

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

impl Value {

    pub fn void(self) -> () {
        match self {
            Value::Void(v) => v,
            _ => panic!("void value expected"),
        }
    }

    pub fn u8(self) -> u8 {
        match self {
            Value::U8(v) => v,
            _ => panic!("u8 value expected"),
        }
    }
    

    pub fn u16(self) -> u16 {
        match self {
            Value::U16(v) => v,
            _ => panic!("u16 value expected"),
        }
    }
    

    pub fn u32(self) -> u32 {
        match self {
            Value::U32(v) => v,
            _ => panic!("u32 value expected"),
        }
    }
    

    pub fn u64(self) -> u64 {
        match self {
            Value::U64(v) => v,
            _ => panic!("u64 value expected"),
        }
    }
    

    pub fn u128(self) -> u128 {
        match self {
            Value::U128(v) => v,
            _ => panic!("u128 value expected"),
        }
    }
    

    pub fn i8(self) -> i8 {
        match self {
            Value::I8(v) => v,
            _ => panic!("i8 value expected"),
        }
    }
    

    pub fn i16(self) -> i16 {
        match self {
            Value::I16(v) => v,
            _ => panic!("i16 value expected"),
        }
    }
    

    pub fn i32(self) -> i32 {
        match self {
            Value::I32(v) => v,
            _ => panic!("i32 value expected"),
        }
    }
    

    pub fn i64(self) -> i64 {
        match self {
            Value::I64(v) => v,
            _ => panic!("i64 value expected"),
        }
    }
    

    pub fn i128(self) -> i128 {
        match self {
            Value::I128(v) => v,
            _ => panic!("i128 value expected"),
        }
    }
    

    pub fn f32(self) -> f32 {
        match self {
            Value::F32(v) => v,
            _ => panic!("f32 value expected"),
        }
    }
    

    pub fn f64(self) -> f64 {
        match self {
            Value::F64(v) => v,
            _ => panic!("f64 value expected"),
        }
    }
    

    pub fn bool(self) -> bool {
        match self {
            Value::Bool(v) => v,
            _ => panic!("bool value expected"),
        }
    }
    

    pub fn byte(self) -> u8 {
        match self {
            Value::Byte(v) => v,
            _ => panic!("byte value expected"),
        }
    }
    

    pub fn char(self) -> char {
        match self {
            Value::Char(v) => v,
            _ => panic!("char value expected"),
        }
    }
    

    pub fn string(self) -> String {
        match self {
            Value::String(v) => v,
            _ => panic!("string value expected"),
        }
    }

    pub fn vec_void(self) -> Vec<()> {
        match self {
            Value::VecVoid(v) => v,
            _ => panic!("Vec<void> value expected"),
        }
    }

    pub fn vec_u8(self) -> Vec<u8> {
        match self {
            Value::VecU8(v) => v,
            _ => panic!("Vec<u8> value expected"),
        }
    }
    

    pub fn vec_u16(self) -> Vec<u16> {
        match self {
            Value::VecU16(v) => v,
            _ => panic!("Vec<u16> value expected"),
        }
    }
    

    pub fn vec_u32(self) -> Vec<u32> {
        match self {
            Value::VecU32(v) => v,
            _ => panic!("Vec<u32> value expected"),
        }
    }
    

    pub fn vec_u64(self) -> Vec<u64> {
        match self {
            Value::VecU64(v) => v,
            _ => panic!("Vec<u64> value expected"),
        }
    }
    

    pub fn vec_u128(self) -> Vec<u128> {
        match self {
            Value::VecU128(v) => v,
            _ => panic!("Vec<u128> value expected"),
        }
    }
    

    pub fn vec_i8(self) -> Vec<i8> {
        match self {
            Value::VecI8(v) => v,
            _ => panic!("Vec<i8> value expected"),
        }
    }
    

    pub fn vec_i16(self) -> Vec<i16> {
        match self {
            Value::VecI16(v) => v,
            _ => panic!("Vec<i16> value expected"),
        }
    }
    

    pub fn vec_i32(self) -> Vec<i32> {
        match self {
            Value::VecI32(v) => v,
            _ => panic!("Vec<i32> value expected"),
        }
    }
    

    pub fn vec_i64(self) -> Vec<i64> {
        match self {
            Value::VecI64(v) => v,
            _ => panic!("Vec<i64> value expected"),
        }
    }
    

    pub fn vec_i128(self) -> Vec<i128> {
        match self {
            Value::VecI128(v) => v,
            _ => panic!("Vec<i128> value expected"),
        }
    }
    

    pub fn vec_f32(self) -> Vec<f32> {
        match self {
            Value::VecF32(v) => v,
            _ => panic!("Vec<f32> value expected"),
        }
    }
    

    pub fn vec_f64(self) -> Vec<f64> {
        match self {
            Value::VecF64(v) => v,
            _ => panic!("Vec<f64> value expected"),
        }
    }
    

    pub fn vec_bool(self) -> Vec<bool> {
        match self {
            Value::VecBool(v) => v,
            _ => panic!("Vec<bool> value expected"),
        }
    }
    

    pub fn vec_byte(self) -> Vec<u8> {
        match self {
            Value::VecByte(v) => v,
            _ => panic!("Vec<byte> value expected"),
        }
    }
    

    pub fn vec_char(self) -> Vec<char> {
        match self {
            Value::VecChar(v) => v,
            _ => panic!("Vec<char> value expected"),
        }
    }
    

    pub fn vec_string(self) -> Vec<String> {
        match self {
            Value::VecString(v) => v,
            _ => panic!("Vec<string> value expected"),
        }
    }
}

impl Display for Value {
    
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        match self {
            Value::Void(_) => write!(f, "()"),
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

            Value::VecVoid(v) => {
                let list: Vec<String> = v.iter().map(|_| format!("()")).collect();
                write!(f, "[{}]", list.join(", "))
            },

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
