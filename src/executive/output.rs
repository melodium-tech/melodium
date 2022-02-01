
use std::sync::Arc;
use super::transmitter::*;
use super::input::Input;
use super::super::logic::descriptor::OutputDescriptor;

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
        todo!()
    }

    pub fn add_input(&self, input: &Input) {
        todo!()
    }

    pub fn close(&self) {
        match self {
            Output::U8(t) => t.close(),
            Output::U16(t) => t.close(),
            Output::U32(t) => t.close(),
            Output::U64(t) => t.close(),
            Output::U128(t) => t.close(),
            Output::I8(t) => t.close(),
            Output::I16(t) => t.close(),
            Output::I32(t) => t.close(),
            Output::I64(t) => t.close(),
            Output::I128(t) => t.close(),
            Output::F32(t) => t.close(),
            Output::F64(t) => t.close(),
            Output::Bool(t) => t.close(),
            Output::Byte(t) => t.close(),
            Output::Char(t) => t.close(),
            Output::String(t) => t.close(),
            Output::VecU8(t) => t.close(),
            Output::VecU16(t) => t.close(),
            Output::VecU32(t) => t.close(),
            Output::VecU64(t) => t.close(),
            Output::VecU128(t) => t.close(),
            Output::VecI8(t) => t.close(),
            Output::VecI16(t) => t.close(),
            Output::VecI32(t) => t.close(),
            Output::VecI64(t) => t.close(),
            Output::VecI128(t) => t.close(),
            Output::VecF32(t) => t.close(),
            Output::VecF64(t) => t.close(),
            Output::VecBool(t) => t.close(),
            Output::VecByte(t) => t.close(),
            Output::VecChar(t) => t.close(),
            Output::VecString(t) => t.close(),
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
