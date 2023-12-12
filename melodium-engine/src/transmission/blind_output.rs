use async_trait::async_trait;
use melodium_common::executive::{Output as ExecutiveOutput, TransmissionError, TransmissionValue};
use melodium_common::executive::{SendResult, Value};

#[derive(Debug, Clone)]
pub struct BlindOutput {}

impl BlindOutput {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ExecutiveOutput for BlindOutput {
    async fn close(&self) {}
    async fn send_many(&self, _data: TransmissionValue) -> SendResult {
        Ok(())
    }
    async fn send_one(&self, _data: Value) -> SendResult {
        Ok(())
    }
    async fn send_one_void(&self, _data: ()) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_void(&self, _data: Vec<()>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_u8(&self, _data: u8) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_u8(&self, _data: Vec<u8>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_u16(&self, _data: u16) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_u16(&self, _data: Vec<u16>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_u32(&self, _data: u32) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_u32(&self, _data: Vec<u32>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_u64(&self, _data: u64) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_u64(&self, _data: Vec<u64>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_u128(&self, _data: u128) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_u128(&self, _data: Vec<u128>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_i8(&self, _data: i8) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_i8(&self, _data: Vec<i8>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_i16(&self, _data: i16) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_i16(&self, _data: Vec<i16>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_i32(&self, _data: i32) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_i32(&self, _data: Vec<i32>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_i64(&self, _data: i64) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_i64(&self, _data: Vec<i64>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_i128(&self, _data: i128) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_i128(&self, _data: Vec<i128>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_f32(&self, _data: f32) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_f32(&self, _data: Vec<f32>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_f64(&self, _data: f64) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_f64(&self, _data: Vec<f64>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_bool(&self, _data: bool) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_bool(&self, _data: Vec<bool>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_byte(&self, _data: u8) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_byte(&self, _data: Vec<u8>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_char(&self, _data: char) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_char(&self, _data: Vec<char>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_string(&self, _data: String) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_string(&self, _data: Vec<String>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_void(&self, _data: Vec<()>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_void(&self, _data: Vec<Vec<()>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_u8(&self, _data: Vec<u8>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_u8(&self, _data: Vec<Vec<u8>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_u16(&self, _data: Vec<u16>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_u16(&self, _data: Vec<Vec<u16>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_u32(&self, _data: Vec<u32>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_u32(&self, _data: Vec<Vec<u32>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_u64(&self, _data: Vec<u64>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_u64(&self, _data: Vec<Vec<u64>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_u128(&self, _data: Vec<u128>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_u128(&self, _data: Vec<Vec<u128>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_i8(&self, _data: Vec<i8>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_i8(&self, _data: Vec<Vec<i8>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_i16(&self, _data: Vec<i16>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_i16(&self, _data: Vec<Vec<i16>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_i32(&self, _data: Vec<i32>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_i32(&self, _data: Vec<Vec<i32>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_i64(&self, _data: Vec<i64>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_i64(&self, _data: Vec<Vec<i64>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_i128(&self, _data: Vec<i128>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_i128(&self, _data: Vec<Vec<i128>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_f32(&self, _data: Vec<f32>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_f32(&self, _data: Vec<Vec<f32>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_f64(&self, _data: Vec<f64>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_f64(&self, _data: Vec<Vec<f64>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_bool(&self, _data: Vec<bool>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_bool(&self, _data: Vec<Vec<bool>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_byte(&self, _data: Vec<u8>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_byte(&self, _data: Vec<Vec<u8>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_char(&self, _data: Vec<char>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_char(&self, _data: Vec<Vec<char>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_one_vec_string(&self, _data: Vec<String>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
    async fn send_vec_string(&self, _data: Vec<Vec<String>>) -> SendResult {
        Err(TransmissionError::NoReceiver)
    }
}
