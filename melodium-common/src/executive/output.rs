use super::{TransmissionValue, Value};
use crate::executive::SendResult;
use async_trait::async_trait;
use core::fmt::Debug;

pub trait Outputs: Debug + Send + Sync {
    fn get(&mut self, output: &str) -> Box<dyn Output>;
}

#[async_trait]
pub trait Output: Debug + Send + Sync {
    async fn close(&self);

    async fn send_many(&self, data: TransmissionValue) -> SendResult;
    async fn send_one(&self, data: Value) -> SendResult;

    async fn send_one_void(&self, data: ()) -> SendResult;
    async fn send_void(&self, data: Vec<()>) -> SendResult;
    async fn send_one_u8(&self, data: u8) -> SendResult;
    async fn send_u8(&self, data: Vec<u8>) -> SendResult;
    async fn send_one_u16(&self, data: u16) -> SendResult;
    async fn send_u16(&self, data: Vec<u16>) -> SendResult;
    async fn send_one_u32(&self, data: u32) -> SendResult;
    async fn send_u32(&self, data: Vec<u32>) -> SendResult;
    async fn send_one_u64(&self, data: u64) -> SendResult;
    async fn send_u64(&self, data: Vec<u64>) -> SendResult;
    async fn send_one_u128(&self, data: u128) -> SendResult;
    async fn send_u128(&self, data: Vec<u128>) -> SendResult;
    async fn send_one_i8(&self, data: i8) -> SendResult;
    async fn send_i8(&self, data: Vec<i8>) -> SendResult;
    async fn send_one_i16(&self, data: i16) -> SendResult;
    async fn send_i16(&self, data: Vec<i16>) -> SendResult;
    async fn send_one_i32(&self, data: i32) -> SendResult;
    async fn send_i32(&self, data: Vec<i32>) -> SendResult;
    async fn send_one_i64(&self, data: i64) -> SendResult;
    async fn send_i64(&self, data: Vec<i64>) -> SendResult;
    async fn send_one_i128(&self, data: i128) -> SendResult;
    async fn send_i128(&self, data: Vec<i128>) -> SendResult;
    async fn send_one_f32(&self, data: f32) -> SendResult;
    async fn send_f32(&self, data: Vec<f32>) -> SendResult;
    async fn send_one_f64(&self, data: f64) -> SendResult;
    async fn send_f64(&self, data: Vec<f64>) -> SendResult;
    async fn send_one_bool(&self, data: bool) -> SendResult;
    async fn send_bool(&self, data: Vec<bool>) -> SendResult;
    async fn send_one_byte(&self, data: u8) -> SendResult;
    async fn send_byte(&self, data: Vec<u8>) -> SendResult;
    async fn send_one_char(&self, data: char) -> SendResult;
    async fn send_char(&self, data: Vec<char>) -> SendResult;
    async fn send_one_string(&self, data: String) -> SendResult;
    async fn send_string(&self, data: Vec<String>) -> SendResult;
    async fn send_one_vec_void(&self, data: Vec<()>) -> SendResult;
    async fn send_vec_void(&self, data: Vec<Vec<()>>) -> SendResult;
    async fn send_one_vec_u8(&self, data: Vec<u8>) -> SendResult;
    async fn send_vec_u8(&self, data: Vec<Vec<u8>>) -> SendResult;
    async fn send_one_vec_u16(&self, data: Vec<u16>) -> SendResult;
    async fn send_vec_u16(&self, data: Vec<Vec<u16>>) -> SendResult;
    async fn send_one_vec_u32(&self, data: Vec<u32>) -> SendResult;
    async fn send_vec_u32(&self, data: Vec<Vec<u32>>) -> SendResult;
    async fn send_one_vec_u64(&self, data: Vec<u64>) -> SendResult;
    async fn send_vec_u64(&self, data: Vec<Vec<u64>>) -> SendResult;
    async fn send_one_vec_u128(&self, data: Vec<u128>) -> SendResult;
    async fn send_vec_u128(&self, data: Vec<Vec<u128>>) -> SendResult;
    async fn send_one_vec_i8(&self, data: Vec<i8>) -> SendResult;
    async fn send_vec_i8(&self, data: Vec<Vec<i8>>) -> SendResult;
    async fn send_one_vec_i16(&self, data: Vec<i16>) -> SendResult;
    async fn send_vec_i16(&self, data: Vec<Vec<i16>>) -> SendResult;
    async fn send_one_vec_i32(&self, data: Vec<i32>) -> SendResult;
    async fn send_vec_i32(&self, data: Vec<Vec<i32>>) -> SendResult;
    async fn send_one_vec_i64(&self, data: Vec<i64>) -> SendResult;
    async fn send_vec_i64(&self, data: Vec<Vec<i64>>) -> SendResult;
    async fn send_one_vec_i128(&self, data: Vec<i128>) -> SendResult;
    async fn send_vec_i128(&self, data: Vec<Vec<i128>>) -> SendResult;
    async fn send_one_vec_f32(&self, data: Vec<f32>) -> SendResult;
    async fn send_vec_f32(&self, data: Vec<Vec<f32>>) -> SendResult;
    async fn send_one_vec_f64(&self, data: Vec<f64>) -> SendResult;
    async fn send_vec_f64(&self, data: Vec<Vec<f64>>) -> SendResult;
    async fn send_one_vec_bool(&self, data: Vec<bool>) -> SendResult;
    async fn send_vec_bool(&self, data: Vec<Vec<bool>>) -> SendResult;
    async fn send_one_vec_byte(&self, data: Vec<u8>) -> SendResult;
    async fn send_vec_byte(&self, data: Vec<Vec<u8>>) -> SendResult;
    async fn send_one_vec_char(&self, data: Vec<char>) -> SendResult;
    async fn send_vec_char(&self, data: Vec<Vec<char>>) -> SendResult;
    async fn send_one_vec_string(&self, data: Vec<String>) -> SendResult;
    async fn send_vec_string(&self, data: Vec<Vec<String>>) -> SendResult;
}
