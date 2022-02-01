
use std::sync::Arc;
use super::transmitter::*;
use super::super::logic::descriptor::InputDescriptor;

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

impl Input {

    pub fn new(descriptor: &InputDescriptor) -> Self {
        todo!()
    }

    pub fn close(&self) {
        match self {
            Input::U8(t) => t.close(),
            Input::U16(t) => t.close(),
            Input::U32(t) => t.close(),
            Input::U64(t) => t.close(),
            Input::U128(t) => t.close(),
            Input::I8(t) => t.close(),
            Input::I16(t) => t.close(),
            Input::I32(t) => t.close(),
            Input::I64(t) => t.close(),
            Input::I128(t) => t.close(),
            Input::F32(t) => t.close(),
            Input::F64(t) => t.close(),
            Input::Bool(t) => t.close(),
            Input::Byte(t) => t.close(),
            Input::Char(t) => t.close(),
            Input::String(t) => t.close(),
            Input::VecU8(t) => t.close(),
            Input::VecU16(t) => t.close(),
            Input::VecU32(t) => t.close(),
            Input::VecU64(t) => t.close(),
            Input::VecU128(t) => t.close(),
            Input::VecI8(t) => t.close(),
            Input::VecI16(t) => t.close(),
            Input::VecI32(t) => t.close(),
            Input::VecI64(t) => t.close(),
            Input::VecI128(t) => t.close(),
            Input::VecF32(t) => t.close(),
            Input::VecF64(t) => t.close(),
            Input::VecBool(t) => t.close(),
            Input::VecByte(t) => t.close(),
            Input::VecChar(t) => t.close(),
            Input::VecString(t) => t.close(),
        }
    }

    pub async fn recv_u8(&self) -> RecvResult<Vec<u8>> {
        match self {
            Input::U8(t) => t.receive_multiple().await,
            _ => panic!("u8 receive transmitter expected"),
        }
    }
    

    pub async fn recv_u16(&self) -> RecvResult<Vec<u16>> {
        match self {
            Input::U16(t) => t.receive_multiple().await,
            _ => panic!("u16 receive transmitter expected"),
        }
    }
    

    pub async fn recv_u32(&self) -> RecvResult<Vec<u32>> {
        match self {
            Input::U32(t) => t.receive_multiple().await,
            _ => panic!("u32 receive transmitter expected"),
        }
    }
    

    pub async fn recv_u64(&self) -> RecvResult<Vec<u64>> {
        match self {
            Input::U64(t) => t.receive_multiple().await,
            _ => panic!("u64 receive transmitter expected"),
        }
    }
    

    pub async fn recv_u128(&self) -> RecvResult<Vec<u128>> {
        match self {
            Input::U128(t) => t.receive_multiple().await,
            _ => panic!("u128 receive transmitter expected"),
        }
    }
    

    pub async fn recv_i8(&self) -> RecvResult<Vec<i8>> {
        match self {
            Input::I8(t) => t.receive_multiple().await,
            _ => panic!("i8 receive transmitter expected"),
        }
    }
    

    pub async fn recv_i16(&self) -> RecvResult<Vec<i16>> {
        match self {
            Input::I16(t) => t.receive_multiple().await,
            _ => panic!("i16 receive transmitter expected"),
        }
    }
    

    pub async fn recv_i32(&self) -> RecvResult<Vec<i32>> {
        match self {
            Input::I32(t) => t.receive_multiple().await,
            _ => panic!("i32 receive transmitter expected"),
        }
    }
    

    pub async fn recv_i64(&self) -> RecvResult<Vec<i64>> {
        match self {
            Input::I64(t) => t.receive_multiple().await,
            _ => panic!("i64 receive transmitter expected"),
        }
    }
    

    pub async fn recv_i128(&self) -> RecvResult<Vec<i128>> {
        match self {
            Input::I128(t) => t.receive_multiple().await,
            _ => panic!("i128 receive transmitter expected"),
        }
    }
    

    pub async fn recv_f32(&self) -> RecvResult<Vec<f32>> {
        match self {
            Input::F32(t) => t.receive_multiple().await,
            _ => panic!("f32 receive transmitter expected"),
        }
    }
    

    pub async fn recv_f64(&self) -> RecvResult<Vec<f64>> {
        match self {
            Input::F64(t) => t.receive_multiple().await,
            _ => panic!("f64 receive transmitter expected"),
        }
    }
    

    pub async fn recv_bool(&self) -> RecvResult<Vec<bool>> {
        match self {
            Input::Bool(t) => t.receive_multiple().await,
            _ => panic!("bool receive transmitter expected"),
        }
    }
    

    pub async fn recv_byte(&self) -> RecvResult<Vec<u8>> {
        match self {
            Input::Byte(t) => t.receive_multiple().await,
            _ => panic!("byte receive transmitter expected"),
        }
    }
    

    pub async fn recv_char(&self) -> RecvResult<Vec<char>> {
        match self {
            Input::Char(t) => t.receive_multiple().await,
            _ => panic!("char receive transmitter expected"),
        }
    }
    

    pub async fn recv_string(&self) -> RecvResult<Vec<String>> {
        match self {
            Input::String(t) => t.receive_multiple().await,
            _ => panic!("string receive transmitter expected"),
        }
    }

    pub async fn recv_vec_u8(&self) -> RecvResult<Vec<Vec<u8>>> {
        match self {
            Input::VecU8(t) => t.receive_multiple().await,
            _ => panic!("Vec<u8> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_u16(&self) -> RecvResult<Vec<Vec<u16>>> {
        match self {
            Input::VecU16(t) => t.receive_multiple().await,
            _ => panic!("Vec<u16> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_u32(&self) -> RecvResult<Vec<Vec<u32>>> {
        match self {
            Input::VecU32(t) => t.receive_multiple().await,
            _ => panic!("Vec<u32> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_u64(&self) -> RecvResult<Vec<Vec<u64>>> {
        match self {
            Input::VecU64(t) => t.receive_multiple().await,
            _ => panic!("Vec<u64> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_u128(&self) -> RecvResult<Vec<Vec<u128>>> {
        match self {
            Input::VecU128(t) => t.receive_multiple().await,
            _ => panic!("Vec<u128> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_i8(&self) -> RecvResult<Vec<Vec<i8>>> {
        match self {
            Input::VecI8(t) => t.receive_multiple().await,
            _ => panic!("Vec<i8> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_i16(&self) -> RecvResult<Vec<Vec<i16>>> {
        match self {
            Input::VecI16(t) => t.receive_multiple().await,
            _ => panic!("Vec<i16> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_i32(&self) -> RecvResult<Vec<Vec<i32>>> {
        match self {
            Input::VecI32(t) => t.receive_multiple().await,
            _ => panic!("Vec<i32> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_i64(&self) -> RecvResult<Vec<Vec<i64>>> {
        match self {
            Input::VecI64(t) => t.receive_multiple().await,
            _ => panic!("Vec<i64> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_i128(&self) -> RecvResult<Vec<Vec<i128>>> {
        match self {
            Input::VecI128(t) => t.receive_multiple().await,
            _ => panic!("Vec<i128> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_f32(&self) -> RecvResult<Vec<Vec<f32>>> {
        match self {
            Input::VecF32(t) => t.receive_multiple().await,
            _ => panic!("Vec<f32> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_f64(&self) -> RecvResult<Vec<Vec<f64>>> {
        match self {
            Input::VecF64(t) => t.receive_multiple().await,
            _ => panic!("Vec<f64> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_bool(&self) -> RecvResult<Vec<Vec<bool>>> {
        match self {
            Input::VecBool(t) => t.receive_multiple().await,
            _ => panic!("Vec<bool> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_byte(&self) -> RecvResult<Vec<Vec<u8>>> {
        match self {
            Input::VecByte(t) => t.receive_multiple().await,
            _ => panic!("Vec<byte> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_char(&self) -> RecvResult<Vec<Vec<char>>> {
        match self {
            Input::VecChar(t) => t.receive_multiple().await,
            _ => panic!("Vec<char> receive transmitter expected"),
        }
    }
    

    pub async fn recv_vec_string(&self) -> RecvResult<Vec<Vec<String>>> {
        match self {
            Input::VecString(t) => t.receive_multiple().await,
            _ => panic!("Vec<string> receive transmitter expected"),
        }
    }


}
