use async_std::channel::{bounded, Receiver, Sender};
use async_std::sync::Mutex as AsyncMutex;
use async_trait::async_trait;
use melodium_common::executive::{
    Input as ExecutiveInput, RecvResult, TransmissionError, TransmissionValue, Value,
};
use std::convert::TryInto;

#[derive(Debug)]
pub struct Input {
    receiver: Receiver<TransmissionValue>,
    sender: Sender<TransmissionValue>,
    buffer: AsyncMutex<Option<TransmissionValue>>,
}

impl Input {
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
impl ExecutiveInput for Input {
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
        unimplemented!()
    }
    async fn recv_vec_u8(&self) -> RecvResult<Vec<Vec<u8>>> {
        unimplemented!()
    }
    async fn recv_vec_u16(&self) -> RecvResult<Vec<Vec<u16>>> {
        unimplemented!()
    }
    async fn recv_vec_u32(&self) -> RecvResult<Vec<Vec<u32>>> {
        unimplemented!()
    }
    async fn recv_vec_u64(&self) -> RecvResult<Vec<Vec<u64>>> {
        unimplemented!()
    }
    async fn recv_vec_u128(&self) -> RecvResult<Vec<Vec<u128>>> {
        unimplemented!()
    }
    async fn recv_vec_i8(&self) -> RecvResult<Vec<Vec<i8>>> {
        unimplemented!()
    }
    async fn recv_vec_i16(&self) -> RecvResult<Vec<Vec<i16>>> {
        unimplemented!()
    }
    async fn recv_vec_i32(&self) -> RecvResult<Vec<Vec<i32>>> {
        unimplemented!()
    }
    async fn recv_vec_i64(&self) -> RecvResult<Vec<Vec<i64>>> {
        unimplemented!()
    }
    async fn recv_vec_i128(&self) -> RecvResult<Vec<Vec<i128>>> {
        unimplemented!()
    }
    async fn recv_vec_f32(&self) -> RecvResult<Vec<Vec<f32>>> {
        unimplemented!()
    }
    async fn recv_vec_f64(&self) -> RecvResult<Vec<Vec<f64>>> {
        unimplemented!()
    }
    async fn recv_vec_bool(&self) -> RecvResult<Vec<Vec<bool>>> {
        unimplemented!()
    }
    async fn recv_vec_byte(&self) -> RecvResult<Vec<Vec<u8>>> {
        unimplemented!()
    }
    async fn recv_vec_char(&self) -> RecvResult<Vec<Vec<char>>> {
        unimplemented!()
    }
    async fn recv_vec_string(&self) -> RecvResult<Vec<Vec<String>>> {
        unimplemented!()
    }
    async fn recv_one_void(&self) -> RecvResult<()> {
        unimplemented!()
    }
    async fn recv_one_u8(&self) -> RecvResult<u8> {
        unimplemented!()
    }
    async fn recv_one_u16(&self) -> RecvResult<u16> {
        unimplemented!()
    }
    async fn recv_one_u32(&self) -> RecvResult<u32> {
        unimplemented!()
    }
    async fn recv_one_u64(&self) -> RecvResult<u64> {
        unimplemented!()
    }
    async fn recv_one_u128(&self) -> RecvResult<u128> {
        unimplemented!()
    }
    async fn recv_one_i8(&self) -> RecvResult<i8> {
        unimplemented!()
    }
    async fn recv_one_i16(&self) -> RecvResult<i16> {
        unimplemented!()
    }
    async fn recv_one_i32(&self) -> RecvResult<i32> {
        unimplemented!()
    }
    async fn recv_one_i64(&self) -> RecvResult<i64> {
        unimplemented!()
    }
    async fn recv_one_i128(&self) -> RecvResult<i128> {
        unimplemented!()
    }
    async fn recv_one_f32(&self) -> RecvResult<f32> {
        unimplemented!()
    }
    async fn recv_one_f64(&self) -> RecvResult<f64> {
        unimplemented!()
    }
    async fn recv_one_bool(&self) -> RecvResult<bool> {
        unimplemented!()
    }
    async fn recv_one_byte(&self) -> RecvResult<u8> {
        unimplemented!()
    }
    async fn recv_one_char(&self) -> RecvResult<char> {
        unimplemented!()
    }
    async fn recv_one_string(&self) -> RecvResult<String> {
        unimplemented!()
    }
    async fn recv_one_vec_void(&self) -> RecvResult<Vec<()>> {
        unimplemented!()
    }
    async fn recv_one_vec_u8(&self) -> RecvResult<Vec<u8>> {
        unimplemented!()
    }
    async fn recv_one_vec_u16(&self) -> RecvResult<Vec<u16>> {
        unimplemented!()
    }
    async fn recv_one_vec_u32(&self) -> RecvResult<Vec<u32>> {
        unimplemented!()
    }
    async fn recv_one_vec_u64(&self) -> RecvResult<Vec<u64>> {
        unimplemented!()
    }
    async fn recv_one_vec_u128(&self) -> RecvResult<Vec<u128>> {
        unimplemented!()
    }
    async fn recv_one_vec_i8(&self) -> RecvResult<Vec<i8>> {
        unimplemented!()
    }
    async fn recv_one_vec_i16(&self) -> RecvResult<Vec<i16>> {
        unimplemented!()
    }
    async fn recv_one_vec_i32(&self) -> RecvResult<Vec<i32>> {
        unimplemented!()
    }
    async fn recv_one_vec_i64(&self) -> RecvResult<Vec<i64>> {
        unimplemented!()
    }
    async fn recv_one_vec_i128(&self) -> RecvResult<Vec<i128>> {
        unimplemented!()
    }
    async fn recv_one_vec_f32(&self) -> RecvResult<Vec<f32>> {
        unimplemented!()
    }
    async fn recv_one_vec_f64(&self) -> RecvResult<Vec<f64>> {
        unimplemented!()
    }
    async fn recv_one_vec_bool(&self) -> RecvResult<Vec<bool>> {
        unimplemented!()
    }
    async fn recv_one_vec_byte(&self) -> RecvResult<Vec<u8>> {
        unimplemented!()
    }
    async fn recv_one_vec_char(&self) -> RecvResult<Vec<char>> {
        unimplemented!()
    }
    async fn recv_one_vec_string(&self) -> RecvResult<Vec<String>> {
        unimplemented!()
    }
}

impl Clone for Input {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver.clone(),
            sender: self.sender.clone(),
            buffer: AsyncMutex::new(None),
        }
    }
}
