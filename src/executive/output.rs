
use std::sync::Arc;
use super::transmitter::*;
use super::input::Input;
use super::super::logic::descriptor::OutputDescriptor;
use super::super::logic::descriptor::datatype::{Type, Structure};

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

impl Output {

    pub fn new(descriptor: &OutputDescriptor) -> Self {
        match descriptor.datatype().structure() {
            Structure::Scalar => {
                match descriptor.datatype().r#type() {
                    Type::U8 => Output::U8(Arc::new(SendTransmitter::new())),
                    Type::U16 => Output::U16(Arc::new(SendTransmitter::new())),
                    Type::U32 => Output::U32(Arc::new(SendTransmitter::new())),
                    Type::U64 => Output::U64(Arc::new(SendTransmitter::new())),
                    Type::U128 => Output::U128(Arc::new(SendTransmitter::new())),
                    Type::I8 => Output::I8(Arc::new(SendTransmitter::new())),
                    Type::I16 => Output::I16(Arc::new(SendTransmitter::new())),
                    Type::I32 => Output::I32(Arc::new(SendTransmitter::new())),
                    Type::I64 => Output::I64(Arc::new(SendTransmitter::new())),
                    Type::I128 => Output::I128(Arc::new(SendTransmitter::new())),
                    Type::F32 => Output::F32(Arc::new(SendTransmitter::new())),
                    Type::F64 => Output::F64(Arc::new(SendTransmitter::new())),
                    Type::Bool => Output::Bool(Arc::new(SendTransmitter::new())),
                    Type::Byte => Output::Byte(Arc::new(SendTransmitter::new())),
                    Type::Char => Output::Char(Arc::new(SendTransmitter::new())),
                    Type::String => Output::String(Arc::new(SendTransmitter::new())),
                }
            },
            Structure::Vector => {
                match descriptor.datatype().r#type() {
                    Type::U8 => Output::VecU8(Arc::new(SendTransmitter::new())),
                    Type::U16 => Output::VecU16(Arc::new(SendTransmitter::new())),
                    Type::U32 => Output::VecU32(Arc::new(SendTransmitter::new())),
                    Type::U64 => Output::VecU64(Arc::new(SendTransmitter::new())),
                    Type::U128 => Output::VecU128(Arc::new(SendTransmitter::new())),
                    Type::I8 => Output::VecI8(Arc::new(SendTransmitter::new())),
                    Type::I16 => Output::VecI16(Arc::new(SendTransmitter::new())),
                    Type::I32 => Output::VecI32(Arc::new(SendTransmitter::new())),
                    Type::I64 => Output::VecI64(Arc::new(SendTransmitter::new())),
                    Type::I128 => Output::VecI128(Arc::new(SendTransmitter::new())),
                    Type::F32 => Output::VecF32(Arc::new(SendTransmitter::new())),
                    Type::F64 => Output::VecF64(Arc::new(SendTransmitter::new())),
                    Type::Bool => Output::VecBool(Arc::new(SendTransmitter::new())),
                    Type::Byte => Output::VecByte(Arc::new(SendTransmitter::new())),
                    Type::Char => Output::VecChar(Arc::new(SendTransmitter::new())),
                    Type::String => Output::VecString(Arc::new(SendTransmitter::new())),
                }
            },
        }
    }

    pub fn add_input(&self, input: &Input) {
        match self {
            Output::U8(st) => {
                match input {
                    Input::U8(it) => st.add_transmitter(it),
                    _ => panic!("u8 send transmitter expected"),
                }
            },
            Output::U16(st) => {
                match input {
                    Input::U16(it) => st.add_transmitter(it),
                    _ => panic!("u16 send transmitter expected"),
                }
            },
            Output::U32(st) => {
                match input {
                    Input::U32(it) => st.add_transmitter(it),
                    _ => panic!("u32 send transmitter expected"),
                }
            },
            Output::U64(st) => {
                match input {
                    Input::U64(it) => st.add_transmitter(it),
                    _ => panic!("u64 send transmitter expected"),
                }
            },
            Output::U128(st) => {
                match input {
                    Input::U128(it) => st.add_transmitter(it),
                    _ => panic!("u128 send transmitter expected"),
                }
            },
            Output::I8(st) => {
                match input {
                    Input::I8(it) => st.add_transmitter(it),
                    _ => panic!("i8 send transmitter expected"),
                }
            },
            Output::I16(st) => {
                match input {
                    Input::I16(it) => st.add_transmitter(it),
                    _ => panic!("i16 send transmitter expected"),
                }
            },
            Output::I32(st) => {
                match input {
                    Input::I32(it) => st.add_transmitter(it),
                    _ => panic!("i32 send transmitter expected"),
                }
            },
            Output::I64(st) => {
                match input {
                    Input::I64(it) => st.add_transmitter(it),
                    _ => panic!("i64 send transmitter expected"),
                }
            },
            Output::I128(st) => {
                match input {
                    Input::I128(it) => st.add_transmitter(it),
                    _ => panic!("i128 send transmitter expected"),
                }
            },
            Output::F32(st) => {
                match input {
                    Input::F32(it) => st.add_transmitter(it),
                    _ => panic!("f32 send transmitter expected"),
                }
            },
            Output::F64(st) => {
                match input {
                    Input::F64(it) => st.add_transmitter(it),
                    _ => panic!("f64 send transmitter expected"),
                }
            },
            Output::Bool(st) => {
                match input {
                    Input::Bool(it) => st.add_transmitter(it),
                    _ => panic!("bool send transmitter expected"),
                }
            },
            Output::Byte(st) => {
                match input {
                    Input::Byte(it) => st.add_transmitter(it),
                    _ => panic!("byte send transmitter expected"),
                }
            },
            Output::Char(st) => {
                match input {
                    Input::Char(it) => st.add_transmitter(it),
                    _ => panic!("char send transmitter expected"),
                }
            },
            Output::String(st) => {
                match input {
                    Input::String(it) => st.add_transmitter(it),
                    _ => panic!("string send transmitter expected"),
                }
            },
            Output::VecU8(st) => {
                match input {
                    Input::VecU8(it) => st.add_transmitter(it),
                    _ => panic!("Vec<u8> send transmitter expected"),
                }
            },
            Output::VecU16(st) => {
                match input {
                    Input::VecU16(it) => st.add_transmitter(it),
                    _ => panic!("Vec<u16> send transmitter expected"),
                }
            },
            Output::VecU32(st) => {
                match input {
                    Input::VecU32(it) => st.add_transmitter(it),
                    _ => panic!("Vec<u32> send transmitter expected"),
                }
            },
            Output::VecU64(st) => {
                match input {
                    Input::VecU64(it) => st.add_transmitter(it),
                    _ => panic!("Vec<u64> send transmitter expected"),
                }
            },
            Output::VecU128(st) => {
                match input {
                    Input::VecU128(it) => st.add_transmitter(it),
                    _ => panic!("Vec<u128> send transmitter expected"),
                }
            },
            Output::VecI8(st) => {
                match input {
                    Input::VecI8(it) => st.add_transmitter(it),
                    _ => panic!("Vec<i8> send transmitter expected"),
                }
            },
            Output::VecI16(st) => {
                match input {
                    Input::VecI16(it) => st.add_transmitter(it),
                    _ => panic!("Vec<i16> send transmitter expected"),
                }
            },
            Output::VecI32(st) => {
                match input {
                    Input::VecI32(it) => st.add_transmitter(it),
                    _ => panic!("Vec<i32> send transmitter expected"),
                }
            },
            Output::VecI64(st) => {
                match input {
                    Input::VecI64(it) => st.add_transmitter(it),
                    _ => panic!("Vec<i64> send transmitter expected"),
                }
            },
            Output::VecI128(st) => {
                match input {
                    Input::VecI128(it) => st.add_transmitter(it),
                    _ => panic!("Vec<i128> send transmitter expected"),
                }
            },
            Output::VecF32(st) => {
                match input {
                    Input::VecF32(it) => st.add_transmitter(it),
                    _ => panic!("Vec<f32> send transmitter expected"),
                }
            },
            Output::VecF64(st) => {
                match input {
                    Input::VecF64(it) => st.add_transmitter(it),
                    _ => panic!("Vec<f64> send transmitter expected"),
                }
            },
            Output::VecBool(st) => {
                match input {
                    Input::VecBool(it) => st.add_transmitter(it),
                    _ => panic!("Vec<bool> send transmitter expected"),
                }
            },
            Output::VecByte(st) => {
                match input {
                    Input::VecByte(it) => st.add_transmitter(it),
                    _ => panic!("Vec<byte> send transmitter expected"),
                }
            },
            Output::VecChar(st) => {
                match input {
                    Input::VecChar(it) => st.add_transmitter(it),
                    _ => panic!("Vec<char> send transmitter expected"),
                }
            },
            Output::VecString(st) => {
                match input {
                    Input::VecString(it) => st.add_transmitter(it),
                    _ => panic!("Vec<string> send transmitter expected"),
                }
            },
        }
    }

    pub async fn close(&self) {
        match self {
            Output::U8(t) => t.close().await,
            Output::U16(t) => t.close().await,
            Output::U32(t) => t.close().await,
            Output::U64(t) => t.close().await,
            Output::U128(t) => t.close().await,
            Output::I8(t) => t.close().await,
            Output::I16(t) => t.close().await,
            Output::I32(t) => t.close().await,
            Output::I64(t) => t.close().await,
            Output::I128(t) => t.close().await,
            Output::F32(t) => t.close().await,
            Output::F64(t) => t.close().await,
            Output::Bool(t) => t.close().await,
            Output::Byte(t) => t.close().await,
            Output::Char(t) => t.close().await,
            Output::String(t) => t.close().await,
            Output::VecU8(t) => t.close().await,
            Output::VecU16(t) => t.close().await,
            Output::VecU32(t) => t.close().await,
            Output::VecU64(t) => t.close().await,
            Output::VecU128(t) => t.close().await,
            Output::VecI8(t) => t.close().await,
            Output::VecI16(t) => t.close().await,
            Output::VecI32(t) => t.close().await,
            Output::VecI64(t) => t.close().await,
            Output::VecI128(t) => t.close().await,
            Output::VecF32(t) => t.close().await,
            Output::VecF64(t) => t.close().await,
            Output::VecBool(t) => t.close().await,
            Output::VecByte(t) => t.close().await,
            Output::VecChar(t) => t.close().await,
            Output::VecString(t) => t.close().await,
        }
    }

    pub async fn send_u8(&self, data: u8) -> SendResult {
        match self {
            Output::U8(t) => t.send(data).await,
            _ => panic!("u8 send transmitter expected"),
        }
    }

    pub async fn send_multiple_u8(&self, data: Vec<u8>) -> SendResult {
        match self {
            Output::U8(t) => t.send_multiple(data).await,
            _ => panic!("u8 send transmitter expected"),
        }
    }
    

    pub async fn send_u16(&self, data: u16) -> SendResult {
        match self {
            Output::U16(t) => t.send(data).await,
            _ => panic!("u16 send transmitter expected"),
        }
    }

    pub async fn send_multiple_u16(&self, data: Vec<u16>) -> SendResult {
        match self {
            Output::U16(t) => t.send_multiple(data).await,
            _ => panic!("u16 send transmitter expected"),
        }
    }
    

    pub async fn send_u32(&self, data: u32) -> SendResult {
        match self {
            Output::U32(t) => t.send(data).await,
            _ => panic!("u32 send transmitter expected"),
        }
    }

    pub async fn send_multiple_u32(&self, data: Vec<u32>) -> SendResult {
        match self {
            Output::U32(t) => t.send_multiple(data).await,
            _ => panic!("u32 send transmitter expected"),
        }
    }
    

    pub async fn send_u64(&self, data: u64) -> SendResult {
        match self {
            Output::U64(t) => t.send(data).await,
            _ => panic!("u64 send transmitter expected"),
        }
    }

    pub async fn send_multiple_u64(&self, data: Vec<u64>) -> SendResult {
        match self {
            Output::U64(t) => t.send_multiple(data).await,
            _ => panic!("u64 send transmitter expected"),
        }
    }
    

    pub async fn send_u128(&self, data: u128) -> SendResult {
        match self {
            Output::U128(t) => t.send(data).await,
            _ => panic!("u128 send transmitter expected"),
        }
    }

    pub async fn send_multiple_u128(&self, data: Vec<u128>) -> SendResult {
        match self {
            Output::U128(t) => t.send_multiple(data).await,
            _ => panic!("u128 send transmitter expected"),
        }
    }
    

    pub async fn send_i8(&self, data: i8) -> SendResult {
        match self {
            Output::I8(t) => t.send(data).await,
            _ => panic!("i8 send transmitter expected"),
        }
    }

    pub async fn send_multiple_i8(&self, data: Vec<i8>) -> SendResult {
        match self {
            Output::I8(t) => t.send_multiple(data).await,
            _ => panic!("i8 send transmitter expected"),
        }
    }
    

    pub async fn send_i16(&self, data: i16) -> SendResult {
        match self {
            Output::I16(t) => t.send(data).await,
            _ => panic!("i16 send transmitter expected"),
        }
    }

    pub async fn send_multiple_i16(&self, data: Vec<i16>) -> SendResult {
        match self {
            Output::I16(t) => t.send_multiple(data).await,
            _ => panic!("i16 send transmitter expected"),
        }
    }
    

    pub async fn send_i32(&self, data: i32) -> SendResult {
        match self {
            Output::I32(t) => t.send(data).await,
            _ => panic!("i32 send transmitter expected"),
        }
    }

    pub async fn send_multiple_i32(&self, data: Vec<i32>) -> SendResult {
        match self {
            Output::I32(t) => t.send_multiple(data).await,
            _ => panic!("i32 send transmitter expected"),
        }
    }
    

    pub async fn send_i64(&self, data: i64) -> SendResult {
        match self {
            Output::I64(t) => t.send(data).await,
            _ => panic!("i64 send transmitter expected"),
        }
    }

    pub async fn send_multiple_i64(&self, data: Vec<i64>) -> SendResult {
        match self {
            Output::I64(t) => t.send_multiple(data).await,
            _ => panic!("i64 send transmitter expected"),
        }
    }
    

    pub async fn send_i128(&self, data: i128) -> SendResult {
        match self {
            Output::I128(t) => t.send(data).await,
            _ => panic!("i128 send transmitter expected"),
        }
    }

    pub async fn send_multiple_i128(&self, data: Vec<i128>) -> SendResult {
        match self {
            Output::I128(t) => t.send_multiple(data).await,
            _ => panic!("i128 send transmitter expected"),
        }
    }
    

    pub async fn send_f32(&self, data: f32) -> SendResult {
        match self {
            Output::F32(t) => t.send(data).await,
            _ => panic!("f32 send transmitter expected"),
        }
    }

    pub async fn send_multiple_f32(&self, data: Vec<f32>) -> SendResult {
        match self {
            Output::F32(t) => t.send_multiple(data).await,
            _ => panic!("f32 send transmitter expected"),
        }
    }
    

    pub async fn send_f64(&self, data: f64) -> SendResult {
        match self {
            Output::F64(t) => t.send(data).await,
            _ => panic!("f64 send transmitter expected"),
        }
    }

    pub async fn send_multiple_f64(&self, data: Vec<f64>) -> SendResult {
        match self {
            Output::F64(t) => t.send_multiple(data).await,
            _ => panic!("f64 send transmitter expected"),
        }
    }
    

    pub async fn send_bool(&self, data: bool) -> SendResult {
        match self {
            Output::Bool(t) => t.send(data).await,
            _ => panic!("bool send transmitter expected"),
        }
    }

    pub async fn send_multiple_bool(&self, data: Vec<bool>) -> SendResult {
        match self {
            Output::Bool(t) => t.send_multiple(data).await,
            _ => panic!("bool send transmitter expected"),
        }
    }
    

    pub async fn send_byte(&self, data: u8) -> SendResult {
        match self {
            Output::Byte(t) => t.send(data).await,
            _ => panic!("byte send transmitter expected"),
        }
    }

    pub async fn send_multiple_byte(&self, data: Vec<u8>) -> SendResult {
        match self {
            Output::Byte(t) => t.send_multiple(data).await,
            _ => panic!("byte send transmitter expected"),
        }
    }
    

    pub async fn send_char(&self, data: char) -> SendResult {
        match self {
            Output::Char(t) => t.send(data).await,
            _ => panic!("char send transmitter expected"),
        }
    }

    pub async fn send_multiple_char(&self, data: Vec<char>) -> SendResult {
        match self {
            Output::Char(t) => t.send_multiple(data).await,
            _ => panic!("char send transmitter expected"),
        }
    }
    

    pub async fn send_string(&self, data: String) -> SendResult {
        match self {
            Output::String(t) => t.send(data).await,
            _ => panic!("string send transmitter expected"),
        }
    }

    pub async fn send_multiple_string(&self, data: Vec<String>) -> SendResult {
        match self {
            Output::String(t) => t.send_multiple(data).await,
            _ => panic!("string send transmitter expected"),
        }
    }


    pub async fn send_vec_u8(&self, data: Vec<u8>) -> SendResult {
        match self {
            Output::VecU8(t) => t.send(data).await,
            _ => panic!("Vec<u8> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_u8(&self, data: Vec<Vec<u8>>) -> SendResult {
        match self {
            Output::VecU8(t) => t.send_multiple(data).await,
            _ => panic!("Vec<u8> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_u16(&self, data: Vec<u16>) -> SendResult {
        match self {
            Output::VecU16(t) => t.send(data).await,
            _ => panic!("Vec<u16> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_u16(&self, data: Vec<Vec<u16>>) -> SendResult {
        match self {
            Output::VecU16(t) => t.send_multiple(data).await,
            _ => panic!("Vec<u16> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_u32(&self, data: Vec<u32>) -> SendResult {
        match self {
            Output::VecU32(t) => t.send(data).await,
            _ => panic!("Vec<u32> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_u32(&self, data: Vec<Vec<u32>>) -> SendResult {
        match self {
            Output::VecU32(t) => t.send_multiple(data).await,
            _ => panic!("Vec<u32> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_u64(&self, data: Vec<u64>) -> SendResult {
        match self {
            Output::VecU64(t) => t.send(data).await,
            _ => panic!("Vec<u64> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_u64(&self, data: Vec<Vec<u64>>) -> SendResult {
        match self {
            Output::VecU64(t) => t.send_multiple(data).await,
            _ => panic!("Vec<u64> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_u128(&self, data: Vec<u128>) -> SendResult {
        match self {
            Output::VecU128(t) => t.send(data).await,
            _ => panic!("Vec<u128> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_u128(&self, data: Vec<Vec<u128>>) -> SendResult {
        match self {
            Output::VecU128(t) => t.send_multiple(data).await,
            _ => panic!("Vec<u128> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_i8(&self, data: Vec<i8>) -> SendResult {
        match self {
            Output::VecI8(t) => t.send(data).await,
            _ => panic!("Vec<i8> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_i8(&self, data: Vec<Vec<i8>>) -> SendResult {
        match self {
            Output::VecI8(t) => t.send_multiple(data).await,
            _ => panic!("Vec<i8> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_i16(&self, data: Vec<i16>) -> SendResult {
        match self {
            Output::VecI16(t) => t.send(data).await,
            _ => panic!("Vec<i16> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_i16(&self, data: Vec<Vec<i16>>) -> SendResult {
        match self {
            Output::VecI16(t) => t.send_multiple(data).await,
            _ => panic!("Vec<i16> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_i32(&self, data: Vec<i32>) -> SendResult {
        match self {
            Output::VecI32(t) => t.send(data).await,
            _ => panic!("Vec<i32> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_i32(&self, data: Vec<Vec<i32>>) -> SendResult {
        match self {
            Output::VecI32(t) => t.send_multiple(data).await,
            _ => panic!("Vec<i32> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_i64(&self, data: Vec<i64>) -> SendResult {
        match self {
            Output::VecI64(t) => t.send(data).await,
            _ => panic!("Vec<i64> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_i64(&self, data: Vec<Vec<i64>>) -> SendResult {
        match self {
            Output::VecI64(t) => t.send_multiple(data).await,
            _ => panic!("Vec<i64> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_i128(&self, data: Vec<i128>) -> SendResult {
        match self {
            Output::VecI128(t) => t.send(data).await,
            _ => panic!("Vec<i128> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_i128(&self, data: Vec<Vec<i128>>) -> SendResult {
        match self {
            Output::VecI128(t) => t.send_multiple(data).await,
            _ => panic!("Vec<i128> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_f32(&self, data: Vec<f32>) -> SendResult {
        match self {
            Output::VecF32(t) => t.send(data).await,
            _ => panic!("Vec<f32> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_f32(&self, data: Vec<Vec<f32>>) -> SendResult {
        match self {
            Output::VecF32(t) => t.send_multiple(data).await,
            _ => panic!("Vec<f32> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_f64(&self, data: Vec<f64>) -> SendResult {
        match self {
            Output::VecF64(t) => t.send(data).await,
            _ => panic!("Vec<f64> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_f64(&self, data: Vec<Vec<f64>>) -> SendResult {
        match self {
            Output::VecF64(t) => t.send_multiple(data).await,
            _ => panic!("Vec<f64> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_bool(&self, data: Vec<bool>) -> SendResult {
        match self {
            Output::VecBool(t) => t.send(data).await,
            _ => panic!("Vec<bool> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_bool(&self, data: Vec<Vec<bool>>) -> SendResult {
        match self {
            Output::VecBool(t) => t.send_multiple(data).await,
            _ => panic!("Vec<bool> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_byte(&self, data: Vec<u8>) -> SendResult {
        match self {
            Output::VecByte(t) => t.send(data).await,
            _ => panic!("Vec<byte> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_byte(&self, data: Vec<Vec<u8>>) -> SendResult {
        match self {
            Output::VecByte(t) => t.send_multiple(data).await,
            _ => panic!("Vec<byte> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_char(&self, data: Vec<char>) -> SendResult {
        match self {
            Output::VecChar(t) => t.send(data).await,
            _ => panic!("Vec<char> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_char(&self, data: Vec<Vec<char>>) -> SendResult {
        match self {
            Output::VecChar(t) => t.send_multiple(data).await,
            _ => panic!("Vec<char> send transmitter expected"),
        }
    }
    

    pub async fn send_vec_string(&self, data: Vec<String>) -> SendResult {
        match self {
            Output::VecString(t) => t.send(data).await,
            _ => panic!("Vec<string> send transmitter expected"),
        }
    }

    pub async fn send_multiple_vec_string(&self, data: Vec<Vec<String>>) -> SendResult {
        match self {
            Output::VecString(t) => t.send_multiple(data).await,
            _ => panic!("Vec<string> send transmitter expected"),
        }
    }

}

impl From<Input> for Output {
    fn from(input: Input) -> Self {
        match input {
            Input::U8(_) => {
                let o = Output::U8(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::U16(_) => {
                let o = Output::U16(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::U32(_) => {
                let o = Output::U32(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::U64(_) => {
                let o = Output::U64(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::U128(_) => {
                let o = Output::U128(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::I8(_) => {
                let o = Output::I8(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::I16(_) => {
                let o = Output::I16(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::I32(_) => {
                let o = Output::I32(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::I64(_) => {
                let o = Output::I64(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::I128(_) => {
                let o = Output::I128(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::F32(_) => {
                let o = Output::F32(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::F64(_) => {
                let o = Output::F64(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::Bool(_) => {
                let o = Output::Bool(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::Byte(_) => {
                let o = Output::Byte(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::Char(_) => {
                let o = Output::Char(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::String(_) => {
                let o = Output::String(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecU8(_) => {
                let o = Output::VecU8(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecU16(_) => {
                let o = Output::VecU16(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecU32(_) => {
                let o = Output::VecU32(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecU64(_) => {
                let o = Output::VecU64(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecU128(_) => {
                let o = Output::VecU128(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecI8(_) => {
                let o = Output::VecI8(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecI16(_) => {
                let o = Output::VecI16(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecI32(_) => {
                let o = Output::VecI32(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecI64(_) => {
                let o = Output::VecI64(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecI128(_) => {
                let o = Output::VecI128(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecF32(_) => {
                let o = Output::VecF32(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecF64(_) => {
                let o = Output::VecF64(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecBool(_) => {
                let o = Output::VecBool(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecByte(_) => {
                let o = Output::VecByte(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecChar(_) => {
                let o = Output::VecChar(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
            Input::VecString(_) => {
                let o = Output::VecString(Arc::new(SendTransmitter::new()));
                o.add_input(&input);
                o
            },
        }
    }
}
