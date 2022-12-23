
use async_trait::async_trait;
use crate::executive::RecvResult;

#[async_trait]
pub trait Input {
    fn close(&self);
    async fn recv_void(&self) -> RecvResult<Vec<()>>;
    async fn recv_u8(&self) -> RecvResult<Vec<u8>>;
    async fn recv_u16(&self) -> RecvResult<Vec<u16>>;
    async fn recv_u32(&self) -> RecvResult<Vec<u32>>;
    async fn recv_u64(&self) -> RecvResult<Vec<u64>>;
    async fn recv_u128(&self) -> RecvResult<Vec<u128>>;
    async fn recv_i8(&self) -> RecvResult<Vec<i8>>;
    async fn recv_i16(&self) -> RecvResult<Vec<i16>>;
    async fn recv_i32(&self) -> RecvResult<Vec<i32>>;
    async fn recv_i64(&self) -> RecvResult<Vec<i64>>;
    async fn recv_i128(&self) -> RecvResult<Vec<i128>>;
    async fn recv_f32(&self) -> RecvResult<Vec<f32>>;
    async fn recv_f64(&self) -> RecvResult<Vec<f64>>;
    async fn recv_bool(&self) -> RecvResult<Vec<bool>>;
    async fn recv_byte(&self) -> RecvResult<Vec<u8>>;
    async fn recv_char(&self) -> RecvResult<Vec<char>>;
    async fn recv_string(&self) -> RecvResult<Vec<String>>;
    async fn recv_vec_void(&self) -> RecvResult<Vec<Vec<()>>>;
    async fn recv_vec_u8(&self) -> RecvResult<Vec<Vec<u8>>>;
    async fn recv_vec_u16(&self) -> RecvResult<Vec<Vec<u16>>>;
    async fn recv_vec_u32(&self) -> RecvResult<Vec<Vec<u32>>>;
    async fn recv_vec_u64(&self) -> RecvResult<Vec<Vec<u64>>>;
    async fn recv_vec_u128(&self) -> RecvResult<Vec<Vec<u128>>>;
    async fn recv_vec_i8(&self) -> RecvResult<Vec<Vec<i8>>>;
    async fn recv_vec_i16(&self) -> RecvResult<Vec<Vec<i16>>>;
    async fn recv_vec_i32(&self) -> RecvResult<Vec<Vec<i32>>>;
    async fn recv_vec_i64(&self) -> RecvResult<Vec<Vec<i64>>>;
    async fn recv_vec_i128(&self) -> RecvResult<Vec<Vec<i128>>>;
    async fn recv_vec_f32(&self) -> RecvResult<Vec<Vec<f32>>>;
    async fn recv_vec_f64(&self) -> RecvResult<Vec<Vec<f64>>>;
    async fn recv_vec_bool(&self) -> RecvResult<Vec<Vec<bool>>>;
    async fn recv_vec_byte(&self) -> RecvResult<Vec<Vec<u8>>>;
    async fn recv_vec_char(&self) -> RecvResult<Vec<Vec<char>>>;
    async fn recv_vec_string(&self) -> RecvResult<Vec<Vec<String>>>;
    async fn recv_one_void(&self) -> RecvResult<()>;
    async fn recv_one_u8(&self) -> RecvResult<u8>;
    async fn recv_one_u16(&self) -> RecvResult<u16>;
    async fn recv_one_u32(&self) -> RecvResult<u32>;
    async fn recv_one_u64(&self) -> RecvResult<u64>;
    async fn recv_one_u128(&self) -> RecvResult<u128>;
    async fn recv_one_i8(&self) -> RecvResult<i8>;
    async fn recv_one_i16(&self) -> RecvResult<i16>;
    async fn recv_one_i32(&self) -> RecvResult<i32>;
    async fn recv_one_i64(&self) -> RecvResult<i64>;
    async fn recv_one_i128(&self) -> RecvResult<i128>;
    async fn recv_one_f32(&self) -> RecvResult<f32>;
    async fn recv_one_f64(&self) -> RecvResult<f64>;
    async fn recv_one_bool(&self) -> RecvResult<bool>;
    async fn recv_one_byte(&self) -> RecvResult<u8>;
    async fn recv_one_char(&self) -> RecvResult<char>;
    async fn recv_one_string(&self) -> RecvResult<String>;
    async fn recv_one_vec_void(&self) -> RecvResult<Vec<()>>;
    async fn recv_one_vec_u8(&self) -> RecvResult<Vec<u8>>;
    async fn recv_one_vec_u16(&self) -> RecvResult<Vec<u16>>;
    async fn recv_one_vec_u32(&self) -> RecvResult<Vec<u32>>;
    async fn recv_one_vec_u64(&self) -> RecvResult<Vec<u64>>;
    async fn recv_one_vec_u128(&self) -> RecvResult<Vec<u128>>;
    async fn recv_one_vec_i8(&self) -> RecvResult<Vec<i8>>;
    async fn recv_one_vec_i16(&self) -> RecvResult<Vec<i16>>;
    async fn recv_one_vec_i32(&self) -> RecvResult<Vec<i32>>;
    async fn recv_one_vec_i64(&self) -> RecvResult<Vec<i64>>;
    async fn recv_one_vec_i128(&self) -> RecvResult<Vec<i128>>;
    async fn recv_one_vec_f32(&self) -> RecvResult<Vec<f32>>;
    async fn recv_one_vec_f64(&self) -> RecvResult<Vec<f64>>;
    async fn recv_one_vec_bool(&self) -> RecvResult<Vec<bool>>;
    async fn recv_one_vec_byte(&self) -> RecvResult<Vec<u8>>;
    async fn recv_one_vec_char(&self) -> RecvResult<Vec<char>>;
    async fn recv_one_vec_string(&self) -> RecvResult<Vec<String>>;
}