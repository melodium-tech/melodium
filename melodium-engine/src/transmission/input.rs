use super::receive_transmitter::RecvTransmitter;
use async_std::channel::{bounded, Receiver, Sender};
use async_std::sync::Mutex as AsyncMutex;
use async_trait::async_trait;
use melodium_common::descriptor::{
    Input as InputDescriptor, Output as OutputDescriptor, Structure, Type,
};
use melodium_common::executive::{
    Input as ExecutiveInput, RecvResult, TransmissionError, TransmissionValue, Value,
};
use std::collections::VecDeque;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct GenericInput {
    receiver: Receiver<TransmissionValue>,
    sender: Sender<TransmissionValue>,
    buffer: AsyncMutex<Option<TransmissionValue>>,
}

impl GenericInput {
    pub fn new() -> Self {
        let (sender, receiver) = bounded(1);
        Self {
            receiver,
            sender,
            buffer: AsyncMutex::new(None),
        }
    }

    pub fn sender(&self) -> &Sender<TransmissionValue> {
        &self.sender
    }
}

#[async_trait]
impl ExecutiveInput for GenericInput {
    fn close(&self) {
        self.receiver.close();
    }

    async fn recv_many(&self) -> RecvResult<TransmissionValue> {
        let mut lock = self.buffer.lock().await;
        if let Some(data) = lock.take() {
            Ok(data)
        } else {
            match self.receiver.recv().await {
                Ok(data) => Ok(data),
                Err(_) => Err(TransmissionError::EverythingClosed),
            }
        }
    }

    async fn recv_one(&self) -> RecvResult<Value> {
        let mut lock = self.buffer.lock().await;
        let value = if let Some(data) = lock.as_mut() {
            data.pop_front().ok_or(TransmissionError::NoData)
        } else {
            match self.receiver.recv().await {
                Ok(mut data) => {
                    let value = data.pop_front().ok_or(TransmissionError::NoData);
                    *lock = Some(data);
                    value
                }
                Err(_) => Err(TransmissionError::EverythingClosed),
            }
        };

        if lock.as_ref().map(|buf| buf.len()).unwrap_or(0) == 0 {
            *lock = None;
        }

        value
    }

    async fn recv_void(&self) -> RecvResult<Vec<()>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_u8(&self) -> RecvResult<Vec<u8>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_u16(&self) -> RecvResult<Vec<u16>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_u32(&self) -> RecvResult<Vec<u32>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_u64(&self) -> RecvResult<Vec<u64>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_u128(&self) -> RecvResult<Vec<u128>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_i8(&self) -> RecvResult<Vec<i8>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_i16(&self) -> RecvResult<Vec<i16>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_i32(&self) -> RecvResult<Vec<i32>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_i64(&self) -> RecvResult<Vec<i64>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_i128(&self) -> RecvResult<Vec<i128>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_f32(&self) -> RecvResult<Vec<f32>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_f64(&self) -> RecvResult<Vec<f64>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_bool(&self) -> RecvResult<Vec<bool>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_byte(&self) -> RecvResult<Vec<u8>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_char(&self) -> RecvResult<Vec<char>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_string(&self) -> RecvResult<Vec<String>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_void(&self) -> RecvResult<Vec<Vec<()>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_u8(&self) -> RecvResult<Vec<Vec<u8>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_u16(&self) -> RecvResult<Vec<Vec<u16>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_u32(&self) -> RecvResult<Vec<Vec<u32>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_u64(&self) -> RecvResult<Vec<Vec<u64>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_u128(&self) -> RecvResult<Vec<Vec<u128>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_i8(&self) -> RecvResult<Vec<Vec<i8>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_i16(&self) -> RecvResult<Vec<Vec<i16>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_i32(&self) -> RecvResult<Vec<Vec<i32>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_i64(&self) -> RecvResult<Vec<Vec<i64>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_i128(&self) -> RecvResult<Vec<Vec<i128>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_f32(&self) -> RecvResult<Vec<Vec<f32>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_f64(&self) -> RecvResult<Vec<Vec<f64>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_bool(&self) -> RecvResult<Vec<Vec<bool>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_byte(&self) -> RecvResult<Vec<Vec<u8>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_char(&self) -> RecvResult<Vec<Vec<char>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_vec_string(&self) -> RecvResult<Vec<Vec<String>>> {
        Ok(self.recv_many().await?.try_into().unwrap())
    }
    async fn recv_one_void(&self) -> RecvResult<()> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_u8(&self) -> RecvResult<u8> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_u16(&self) -> RecvResult<u16> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_u32(&self) -> RecvResult<u32> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_u64(&self) -> RecvResult<u64> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_u128(&self) -> RecvResult<u128> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_i8(&self) -> RecvResult<i8> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_i16(&self) -> RecvResult<i16> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_i32(&self) -> RecvResult<i32> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_i64(&self) -> RecvResult<i64> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_i128(&self) -> RecvResult<i128> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_f32(&self) -> RecvResult<f32> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_f64(&self) -> RecvResult<f64> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_bool(&self) -> RecvResult<bool> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_byte(&self) -> RecvResult<u8> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_char(&self) -> RecvResult<char> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_string(&self) -> RecvResult<String> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_void(&self) -> RecvResult<Vec<()>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_u8(&self) -> RecvResult<Vec<u8>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_u16(&self) -> RecvResult<Vec<u16>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_u32(&self) -> RecvResult<Vec<u32>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_u64(&self) -> RecvResult<Vec<u64>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_u128(&self) -> RecvResult<Vec<u128>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_i8(&self) -> RecvResult<Vec<i8>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_i16(&self) -> RecvResult<Vec<i16>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_i32(&self) -> RecvResult<Vec<i32>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_i64(&self) -> RecvResult<Vec<i64>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_i128(&self) -> RecvResult<Vec<i128>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_f32(&self) -> RecvResult<Vec<f32>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_f64(&self) -> RecvResult<Vec<f64>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_bool(&self) -> RecvResult<Vec<bool>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_byte(&self) -> RecvResult<Vec<u8>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_char(&self) -> RecvResult<Vec<char>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
    async fn recv_one_vec_string(&self) -> RecvResult<Vec<String>> {
        Ok(self.recv_one().await?.try_into().unwrap())
    }
}

impl Clone for GenericInput {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver.clone(),
            sender: self.sender.clone(),
            buffer: AsyncMutex::new(None),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Input {
    Void(Arc<RecvTransmitter<()>>),

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

    VecVoid(Arc<RecvTransmitter<Vec<()>>>),

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
        match descriptor.datatype().structure() {
            Structure::Scalar => match descriptor.datatype().r#type() {
                Type::Void => Input::Void(Arc::new(RecvTransmitter::new())),
                Type::U8 => Input::U8(Arc::new(RecvTransmitter::new())),
                Type::U16 => Input::U16(Arc::new(RecvTransmitter::new())),
                Type::U32 => Input::U32(Arc::new(RecvTransmitter::new())),
                Type::U64 => Input::U64(Arc::new(RecvTransmitter::new())),
                Type::U128 => Input::U128(Arc::new(RecvTransmitter::new())),
                Type::I8 => Input::I8(Arc::new(RecvTransmitter::new())),
                Type::I16 => Input::I16(Arc::new(RecvTransmitter::new())),
                Type::I32 => Input::I32(Arc::new(RecvTransmitter::new())),
                Type::I64 => Input::I64(Arc::new(RecvTransmitter::new())),
                Type::I128 => Input::I128(Arc::new(RecvTransmitter::new())),
                Type::F32 => Input::F32(Arc::new(RecvTransmitter::new())),
                Type::F64 => Input::F64(Arc::new(RecvTransmitter::new())),
                Type::Bool => Input::Bool(Arc::new(RecvTransmitter::new())),
                Type::Byte => Input::Byte(Arc::new(RecvTransmitter::new())),
                Type::Char => Input::Char(Arc::new(RecvTransmitter::new())),
                Type::String => Input::String(Arc::new(RecvTransmitter::new())),
            },
            Structure::Vector => match descriptor.datatype().r#type() {
                Type::Void => Input::VecVoid(Arc::new(RecvTransmitter::new())),
                Type::U8 => Input::VecU8(Arc::new(RecvTransmitter::new())),
                Type::U16 => Input::VecU16(Arc::new(RecvTransmitter::new())),
                Type::U32 => Input::VecU32(Arc::new(RecvTransmitter::new())),
                Type::U64 => Input::VecU64(Arc::new(RecvTransmitter::new())),
                Type::U128 => Input::VecU128(Arc::new(RecvTransmitter::new())),
                Type::I8 => Input::VecI8(Arc::new(RecvTransmitter::new())),
                Type::I16 => Input::VecI16(Arc::new(RecvTransmitter::new())),
                Type::I32 => Input::VecI32(Arc::new(RecvTransmitter::new())),
                Type::I64 => Input::VecI64(Arc::new(RecvTransmitter::new())),
                Type::I128 => Input::VecI128(Arc::new(RecvTransmitter::new())),
                Type::F32 => Input::VecF32(Arc::new(RecvTransmitter::new())),
                Type::F64 => Input::VecF64(Arc::new(RecvTransmitter::new())),
                Type::Bool => Input::VecBool(Arc::new(RecvTransmitter::new())),
                Type::Byte => Input::VecByte(Arc::new(RecvTransmitter::new())),
                Type::Char => Input::VecChar(Arc::new(RecvTransmitter::new())),
                Type::String => Input::VecString(Arc::new(RecvTransmitter::new())),
            },
        }
    }

    pub fn from_output(descriptor: &OutputDescriptor) -> Self {
        match descriptor.datatype().structure() {
            Structure::Scalar => match descriptor.datatype().r#type() {
                Type::Void => Input::Void(Arc::new(RecvTransmitter::new())),
                Type::U8 => Input::U8(Arc::new(RecvTransmitter::new())),
                Type::U16 => Input::U16(Arc::new(RecvTransmitter::new())),
                Type::U32 => Input::U32(Arc::new(RecvTransmitter::new())),
                Type::U64 => Input::U64(Arc::new(RecvTransmitter::new())),
                Type::U128 => Input::U128(Arc::new(RecvTransmitter::new())),
                Type::I8 => Input::I8(Arc::new(RecvTransmitter::new())),
                Type::I16 => Input::I16(Arc::new(RecvTransmitter::new())),
                Type::I32 => Input::I32(Arc::new(RecvTransmitter::new())),
                Type::I64 => Input::I64(Arc::new(RecvTransmitter::new())),
                Type::I128 => Input::I128(Arc::new(RecvTransmitter::new())),
                Type::F32 => Input::F32(Arc::new(RecvTransmitter::new())),
                Type::F64 => Input::F64(Arc::new(RecvTransmitter::new())),
                Type::Bool => Input::Bool(Arc::new(RecvTransmitter::new())),
                Type::Byte => Input::Byte(Arc::new(RecvTransmitter::new())),
                Type::Char => Input::Char(Arc::new(RecvTransmitter::new())),
                Type::String => Input::String(Arc::new(RecvTransmitter::new())),
            },
            Structure::Vector => match descriptor.datatype().r#type() {
                Type::Void => Input::VecVoid(Arc::new(RecvTransmitter::new())),
                Type::U8 => Input::VecU8(Arc::new(RecvTransmitter::new())),
                Type::U16 => Input::VecU16(Arc::new(RecvTransmitter::new())),
                Type::U32 => Input::VecU32(Arc::new(RecvTransmitter::new())),
                Type::U64 => Input::VecU64(Arc::new(RecvTransmitter::new())),
                Type::U128 => Input::VecU128(Arc::new(RecvTransmitter::new())),
                Type::I8 => Input::VecI8(Arc::new(RecvTransmitter::new())),
                Type::I16 => Input::VecI16(Arc::new(RecvTransmitter::new())),
                Type::I32 => Input::VecI32(Arc::new(RecvTransmitter::new())),
                Type::I64 => Input::VecI64(Arc::new(RecvTransmitter::new())),
                Type::I128 => Input::VecI128(Arc::new(RecvTransmitter::new())),
                Type::F32 => Input::VecF32(Arc::new(RecvTransmitter::new())),
                Type::F64 => Input::VecF64(Arc::new(RecvTransmitter::new())),
                Type::Bool => Input::VecBool(Arc::new(RecvTransmitter::new())),
                Type::Byte => Input::VecByte(Arc::new(RecvTransmitter::new())),
                Type::Char => Input::VecChar(Arc::new(RecvTransmitter::new())),
                Type::String => Input::VecString(Arc::new(RecvTransmitter::new())),
            },
        }
    }
}

#[async_trait]
impl ExecutiveInput for Input {
    fn close(&self) {
        match self {
            Input::Void(t) => t.close(),
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
            Input::VecVoid(t) => t.close(),
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

    async fn recv_many(&self) -> RecvResult<TransmissionValue> {
        Ok(TransmissionValue::Void(VecDeque::new()))
    }

    async fn recv_one(&self) -> RecvResult<Value> {
        Ok(Value::Void(()))
    }

    async fn recv_void(&self) -> RecvResult<Vec<()>> {
        match self {
            Input::Void(t) => t.receive_multiple().await,
            _ => panic!("void receive transmitter expected"),
        }
    }

    async fn recv_u8(&self) -> RecvResult<Vec<u8>> {
        match self {
            Input::U8(t) => t.receive_multiple().await,
            _ => panic!("u8 receive transmitter expected"),
        }
    }

    async fn recv_u16(&self) -> RecvResult<Vec<u16>> {
        match self {
            Input::U16(t) => t.receive_multiple().await,
            _ => panic!("u16 receive transmitter expected"),
        }
    }

    async fn recv_u32(&self) -> RecvResult<Vec<u32>> {
        match self {
            Input::U32(t) => t.receive_multiple().await,
            _ => panic!("u32 receive transmitter expected"),
        }
    }

    async fn recv_u64(&self) -> RecvResult<Vec<u64>> {
        match self {
            Input::U64(t) => t.receive_multiple().await,
            _ => panic!("u64 receive transmitter expected"),
        }
    }

    async fn recv_u128(&self) -> RecvResult<Vec<u128>> {
        match self {
            Input::U128(t) => t.receive_multiple().await,
            _ => panic!("u128 receive transmitter expected"),
        }
    }

    async fn recv_i8(&self) -> RecvResult<Vec<i8>> {
        match self {
            Input::I8(t) => t.receive_multiple().await,
            _ => panic!("i8 receive transmitter expected"),
        }
    }

    async fn recv_i16(&self) -> RecvResult<Vec<i16>> {
        match self {
            Input::I16(t) => t.receive_multiple().await,
            _ => panic!("i16 receive transmitter expected"),
        }
    }

    async fn recv_i32(&self) -> RecvResult<Vec<i32>> {
        match self {
            Input::I32(t) => t.receive_multiple().await,
            _ => panic!("i32 receive transmitter expected"),
        }
    }

    async fn recv_i64(&self) -> RecvResult<Vec<i64>> {
        match self {
            Input::I64(t) => t.receive_multiple().await,
            _ => panic!("i64 receive transmitter expected"),
        }
    }

    async fn recv_i128(&self) -> RecvResult<Vec<i128>> {
        match self {
            Input::I128(t) => t.receive_multiple().await,
            _ => panic!("i128 receive transmitter expected"),
        }
    }

    async fn recv_f32(&self) -> RecvResult<Vec<f32>> {
        match self {
            Input::F32(t) => t.receive_multiple().await,
            _ => panic!("f32 receive transmitter expected"),
        }
    }

    async fn recv_f64(&self) -> RecvResult<Vec<f64>> {
        match self {
            Input::F64(t) => t.receive_multiple().await,
            _ => panic!("f64 receive transmitter expected"),
        }
    }

    async fn recv_bool(&self) -> RecvResult<Vec<bool>> {
        match self {
            Input::Bool(t) => t.receive_multiple().await,
            _ => panic!("bool receive transmitter expected"),
        }
    }

    async fn recv_byte(&self) -> RecvResult<Vec<u8>> {
        match self {
            Input::Byte(t) => t.receive_multiple().await,
            _ => panic!("byte receive transmitter expected"),
        }
    }

    async fn recv_char(&self) -> RecvResult<Vec<char>> {
        match self {
            Input::Char(t) => t.receive_multiple().await,
            _ => panic!("char receive transmitter expected"),
        }
    }

    async fn recv_string(&self) -> RecvResult<Vec<String>> {
        match self {
            Input::String(t) => t.receive_multiple().await,
            _ => panic!("string receive transmitter expected"),
        }
    }

    async fn recv_vec_void(&self) -> RecvResult<Vec<Vec<()>>> {
        match self {
            Input::VecVoid(t) => t.receive_multiple().await,
            _ => panic!("Vec<void> receive transmitter expected"),
        }
    }

    async fn recv_vec_u8(&self) -> RecvResult<Vec<Vec<u8>>> {
        match self {
            Input::VecU8(t) => t.receive_multiple().await,
            _ => panic!("Vec<u8> receive transmitter expected"),
        }
    }

    async fn recv_vec_u16(&self) -> RecvResult<Vec<Vec<u16>>> {
        match self {
            Input::VecU16(t) => t.receive_multiple().await,
            _ => panic!("Vec<u16> receive transmitter expected"),
        }
    }

    async fn recv_vec_u32(&self) -> RecvResult<Vec<Vec<u32>>> {
        match self {
            Input::VecU32(t) => t.receive_multiple().await,
            _ => panic!("Vec<u32> receive transmitter expected"),
        }
    }

    async fn recv_vec_u64(&self) -> RecvResult<Vec<Vec<u64>>> {
        match self {
            Input::VecU64(t) => t.receive_multiple().await,
            _ => panic!("Vec<u64> receive transmitter expected"),
        }
    }

    async fn recv_vec_u128(&self) -> RecvResult<Vec<Vec<u128>>> {
        match self {
            Input::VecU128(t) => t.receive_multiple().await,
            _ => panic!("Vec<u128> receive transmitter expected"),
        }
    }

    async fn recv_vec_i8(&self) -> RecvResult<Vec<Vec<i8>>> {
        match self {
            Input::VecI8(t) => t.receive_multiple().await,
            _ => panic!("Vec<i8> receive transmitter expected"),
        }
    }

    async fn recv_vec_i16(&self) -> RecvResult<Vec<Vec<i16>>> {
        match self {
            Input::VecI16(t) => t.receive_multiple().await,
            _ => panic!("Vec<i16> receive transmitter expected"),
        }
    }

    async fn recv_vec_i32(&self) -> RecvResult<Vec<Vec<i32>>> {
        match self {
            Input::VecI32(t) => t.receive_multiple().await,
            _ => panic!("Vec<i32> receive transmitter expected"),
        }
    }

    async fn recv_vec_i64(&self) -> RecvResult<Vec<Vec<i64>>> {
        match self {
            Input::VecI64(t) => t.receive_multiple().await,
            _ => panic!("Vec<i64> receive transmitter expected"),
        }
    }

    async fn recv_vec_i128(&self) -> RecvResult<Vec<Vec<i128>>> {
        match self {
            Input::VecI128(t) => t.receive_multiple().await,
            _ => panic!("Vec<i128> receive transmitter expected"),
        }
    }

    async fn recv_vec_f32(&self) -> RecvResult<Vec<Vec<f32>>> {
        match self {
            Input::VecF32(t) => t.receive_multiple().await,
            _ => panic!("Vec<f32> receive transmitter expected"),
        }
    }

    async fn recv_vec_f64(&self) -> RecvResult<Vec<Vec<f64>>> {
        match self {
            Input::VecF64(t) => t.receive_multiple().await,
            _ => panic!("Vec<f64> receive transmitter expected"),
        }
    }

    async fn recv_vec_bool(&self) -> RecvResult<Vec<Vec<bool>>> {
        match self {
            Input::VecBool(t) => t.receive_multiple().await,
            _ => panic!("Vec<bool> receive transmitter expected"),
        }
    }

    async fn recv_vec_byte(&self) -> RecvResult<Vec<Vec<u8>>> {
        match self {
            Input::VecByte(t) => t.receive_multiple().await,
            _ => panic!("Vec<byte> receive transmitter expected"),
        }
    }

    async fn recv_vec_char(&self) -> RecvResult<Vec<Vec<char>>> {
        match self {
            Input::VecChar(t) => t.receive_multiple().await,
            _ => panic!("Vec<char> receive transmitter expected"),
        }
    }

    async fn recv_vec_string(&self) -> RecvResult<Vec<Vec<String>>> {
        match self {
            Input::VecString(t) => t.receive_multiple().await,
            _ => panic!("Vec<string> receive transmitter expected"),
        }
    }

    async fn recv_one_void(&self) -> RecvResult<()> {
        match self {
            Input::Void(t) => t.receive_one().await,
            _ => panic!("void receive transmitter expected"),
        }
    }

    async fn recv_one_u8(&self) -> RecvResult<u8> {
        match self {
            Input::U8(t) => t.receive_one().await,
            _ => panic!("u8 receive transmitter expected"),
        }
    }

    async fn recv_one_u16(&self) -> RecvResult<u16> {
        match self {
            Input::U16(t) => t.receive_one().await,
            _ => panic!("u16 receive transmitter expected"),
        }
    }

    async fn recv_one_u32(&self) -> RecvResult<u32> {
        match self {
            Input::U32(t) => t.receive_one().await,
            _ => panic!("u32 receive transmitter expected"),
        }
    }

    async fn recv_one_u64(&self) -> RecvResult<u64> {
        match self {
            Input::U64(t) => t.receive_one().await,
            _ => panic!("u64 receive transmitter expected"),
        }
    }

    async fn recv_one_u128(&self) -> RecvResult<u128> {
        match self {
            Input::U128(t) => t.receive_one().await,
            _ => panic!("u128 receive transmitter expected"),
        }
    }

    async fn recv_one_i8(&self) -> RecvResult<i8> {
        match self {
            Input::I8(t) => t.receive_one().await,
            _ => panic!("i8 receive transmitter expected"),
        }
    }

    async fn recv_one_i16(&self) -> RecvResult<i16> {
        match self {
            Input::I16(t) => t.receive_one().await,
            _ => panic!("i16 receive transmitter expected"),
        }
    }

    async fn recv_one_i32(&self) -> RecvResult<i32> {
        match self {
            Input::I32(t) => t.receive_one().await,
            _ => panic!("i32 receive transmitter expected"),
        }
    }

    async fn recv_one_i64(&self) -> RecvResult<i64> {
        match self {
            Input::I64(t) => t.receive_one().await,
            _ => panic!("i64 receive transmitter expected"),
        }
    }

    async fn recv_one_i128(&self) -> RecvResult<i128> {
        match self {
            Input::I128(t) => t.receive_one().await,
            _ => panic!("i128 receive transmitter expected"),
        }
    }

    async fn recv_one_f32(&self) -> RecvResult<f32> {
        match self {
            Input::F32(t) => t.receive_one().await,
            _ => panic!("f32 receive transmitter expected"),
        }
    }

    async fn recv_one_f64(&self) -> RecvResult<f64> {
        match self {
            Input::F64(t) => t.receive_one().await,
            _ => panic!("f64 receive transmitter expected"),
        }
    }

    async fn recv_one_bool(&self) -> RecvResult<bool> {
        match self {
            Input::Bool(t) => t.receive_one().await,
            _ => panic!("bool receive transmitter expected"),
        }
    }

    async fn recv_one_byte(&self) -> RecvResult<u8> {
        match self {
            Input::Byte(t) => t.receive_one().await,
            _ => panic!("byte receive transmitter expected"),
        }
    }

    async fn recv_one_char(&self) -> RecvResult<char> {
        match self {
            Input::Char(t) => t.receive_one().await,
            _ => panic!("char receive transmitter expected"),
        }
    }

    async fn recv_one_string(&self) -> RecvResult<String> {
        match self {
            Input::String(t) => t.receive_one().await,
            _ => panic!("string receive transmitter expected"),
        }
    }

    async fn recv_one_vec_void(&self) -> RecvResult<Vec<()>> {
        match self {
            Input::VecVoid(t) => t.receive_one().await,
            _ => panic!("Vec<void> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_u8(&self) -> RecvResult<Vec<u8>> {
        match self {
            Input::VecU8(t) => t.receive_one().await,
            _ => panic!("Vec<u8> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_u16(&self) -> RecvResult<Vec<u16>> {
        match self {
            Input::VecU16(t) => t.receive_one().await,
            _ => panic!("Vec<u16> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_u32(&self) -> RecvResult<Vec<u32>> {
        match self {
            Input::VecU32(t) => t.receive_one().await,
            _ => panic!("Vec<u32> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_u64(&self) -> RecvResult<Vec<u64>> {
        match self {
            Input::VecU64(t) => t.receive_one().await,
            _ => panic!("Vec<u64> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_u128(&self) -> RecvResult<Vec<u128>> {
        match self {
            Input::VecU128(t) => t.receive_one().await,
            _ => panic!("Vec<u128> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_i8(&self) -> RecvResult<Vec<i8>> {
        match self {
            Input::VecI8(t) => t.receive_one().await,
            _ => panic!("Vec<i8> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_i16(&self) -> RecvResult<Vec<i16>> {
        match self {
            Input::VecI16(t) => t.receive_one().await,
            _ => panic!("Vec<i16> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_i32(&self) -> RecvResult<Vec<i32>> {
        match self {
            Input::VecI32(t) => t.receive_one().await,
            _ => panic!("Vec<i32> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_i64(&self) -> RecvResult<Vec<i64>> {
        match self {
            Input::VecI64(t) => t.receive_one().await,
            _ => panic!("Vec<i64> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_i128(&self) -> RecvResult<Vec<i128>> {
        match self {
            Input::VecI128(t) => t.receive_one().await,
            _ => panic!("Vec<i128> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_f32(&self) -> RecvResult<Vec<f32>> {
        match self {
            Input::VecF32(t) => t.receive_one().await,
            _ => panic!("Vec<f32> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_f64(&self) -> RecvResult<Vec<f64>> {
        match self {
            Input::VecF64(t) => t.receive_one().await,
            _ => panic!("Vec<f64> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_bool(&self) -> RecvResult<Vec<bool>> {
        match self {
            Input::VecBool(t) => t.receive_one().await,
            _ => panic!("Vec<bool> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_byte(&self) -> RecvResult<Vec<u8>> {
        match self {
            Input::VecByte(t) => t.receive_one().await,
            _ => panic!("Vec<byte> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_char(&self) -> RecvResult<Vec<char>> {
        match self {
            Input::VecChar(t) => t.receive_one().await,
            _ => panic!("Vec<char> receive transmitter expected"),
        }
    }

    async fn recv_one_vec_string(&self) -> RecvResult<Vec<String>> {
        match self {
            Input::VecString(t) => t.receive_one().await,
            _ => panic!("Vec<string> receive transmitter expected"),
        }
    }
}
