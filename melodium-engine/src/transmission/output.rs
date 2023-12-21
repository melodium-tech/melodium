use crate::transmission::Input;
use async_std::channel::Sender;
use async_std::sync::Mutex as AsyncMutex;
use async_trait::async_trait;
use core::sync::atomic::{AtomicUsize, Ordering};
use melodium_common::executive::{
    Output as ExecutiveOutput, SendResult, TransmissionError, TransmissionValue, Value,
};
use std::sync::{Arc, Mutex};

const LIMIT: usize = 2usize.pow(20);

#[derive(Debug)]
pub struct Output {
    senders: Mutex<Arc<Vec<Sender<TransmissionValue>>>>,
    count_receivers: AtomicUsize,
    buffer: AsyncMutex<Option<TransmissionValue>>,
}

impl Output {
    pub fn new() -> Self {
        Self {
            senders: Mutex::new(Arc::new(Vec::new())),
            count_receivers: AtomicUsize::new(0),
            buffer: AsyncMutex::new(None),
        }
    }

    pub fn add_transmission(&self, inputs: &Vec<Input>) {
        let mut senders = self.senders.lock().unwrap();
        let count = inputs.len();
        // An output is not supposed to have transmission added while it is already in use,
        // so get_mut on Arc is doable.
        if let Some(senders) = Arc::get_mut(&mut senders) {
            for input in inputs {
                senders.push(input.sender().clone())
            }
            self.count_receivers.fetch_add(count, Ordering::Relaxed);
        }
    }

    async fn check_send(&self, force: bool) -> SendResult {
        let buffer_len = self
            .buffer
            .lock()
            .await
            .as_ref()
            .map(|buf| buf.len())
            .unwrap_or(0);

        if buffer_len >= LIMIT || (force && buffer_len > 0) {
            match self.count_receivers.load(Ordering::Relaxed) {
                0 => Err(TransmissionError::NoReceiver),
                1 => {
                    let senders = Arc::clone(&self.senders.lock().unwrap());
                    if let Some(sender) = senders.first() {
                        // We can unwrap the `take` because buffer_len must be > 0, so buffer have value.
                        let data = self.buffer.lock().await.take().unwrap();
                        match sender.send(data).await {
                            Ok(_) => Ok(()),
                            Err(_) => Err(TransmissionError::EverythingClosed),
                        }
                    } else {
                        Err(TransmissionError::NoReceiver)
                    }
                }
                _ => {
                    let mut statuses = Vec::new();
                    let senders = Arc::clone(&self.senders.lock().unwrap());

                    // We can unwrap the `take` because buffer_len must be > 0, so buffer have value.
                    let data = self.buffer.lock().await.take().unwrap();
                    for sender in senders.iter() {
                        statuses.push(match sender.send(data.clone()).await {
                            Ok(()) => true,
                            Err(_) => false,
                        });
                    }

                    if let Some(_) = statuses.iter().find(|s| **s) {
                        Ok(())
                    } else {
                        Err(TransmissionError::EverythingClosed)
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl ExecutiveOutput for Output {
    async fn close(&self) {
        let _ = self.check_send(true).await;
        self.senders.lock().unwrap().iter().for_each(|s| {
            s.close();
        });
    }

    async fn send_many(&self, data: TransmissionValue) -> SendResult {
        {
            let mut lock = self.buffer.lock().await;
            if let Some(buf) = lock.as_mut() {
                buf.append(data);
            } else {
                *lock = Some(data);
            }
        }
        self.check_send(false).await
    }
    async fn send_one(&self, data: Value) -> SendResult {
        {
            let mut lock = self.buffer.lock().await;
            if let Some(buf) = lock.as_mut() {
                buf.push(data);
            } else {
                *lock = Some(TransmissionValue::new(data));
            }
        }
        self.check_send(false).await
    }
    async fn send_one_void(&self, data: ()) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_void(&self, data: Vec<()>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_u8(&self, data: u8) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_u8(&self, data: Vec<u8>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_u16(&self, data: u16) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_u16(&self, data: Vec<u16>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_u32(&self, data: u32) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_u32(&self, data: Vec<u32>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_u64(&self, data: u64) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_u64(&self, data: Vec<u64>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_u128(&self, data: u128) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_u128(&self, data: Vec<u128>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_i8(&self, data: i8) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_i8(&self, data: Vec<i8>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_i16(&self, data: i16) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_i16(&self, data: Vec<i16>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_i32(&self, data: i32) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_i32(&self, data: Vec<i32>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_i64(&self, data: i64) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_i64(&self, data: Vec<i64>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_i128(&self, data: i128) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_i128(&self, data: Vec<i128>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_f32(&self, data: f32) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_f32(&self, data: Vec<f32>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_f64(&self, data: f64) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_f64(&self, data: Vec<f64>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_bool(&self, data: bool) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_bool(&self, data: Vec<bool>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_byte(&self, data: u8) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_byte(&self, data: Vec<u8>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_char(&self, data: char) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_char(&self, data: Vec<char>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_string(&self, data: String) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_string(&self, data: Vec<String>) -> SendResult {
        self.send_many(data.into()).await
    }
    async fn send_one_vec_void(&self, data: Vec<()>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_void(&self, data: Vec<Vec<()>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_u8(&self, data: Vec<u8>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_u8(&self, data: Vec<Vec<u8>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_u16(&self, data: Vec<u16>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_u16(&self, data: Vec<Vec<u16>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_u32(&self, data: Vec<u32>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_u32(&self, data: Vec<Vec<u32>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_u64(&self, data: Vec<u64>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_u64(&self, data: Vec<Vec<u64>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_u128(&self, data: Vec<u128>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_u128(&self, data: Vec<Vec<u128>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_i8(&self, data: Vec<i8>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_i8(&self, data: Vec<Vec<i8>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_i16(&self, data: Vec<i16>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_i16(&self, data: Vec<Vec<i16>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_i32(&self, data: Vec<i32>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_i32(&self, data: Vec<Vec<i32>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_i64(&self, data: Vec<i64>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_i64(&self, data: Vec<Vec<i64>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_i128(&self, data: Vec<i128>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_i128(&self, data: Vec<Vec<i128>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_f32(&self, data: Vec<f32>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_f32(&self, data: Vec<Vec<f32>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_f64(&self, data: Vec<f64>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_f64(&self, data: Vec<Vec<f64>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_bool(&self, data: Vec<bool>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_bool(&self, data: Vec<Vec<bool>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_byte(&self, data: Vec<u8>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_byte(&self, data: Vec<Vec<u8>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_char(&self, data: Vec<char>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_char(&self, data: Vec<Vec<char>>) -> SendResult {
        unimplemented!()
    }
    async fn send_one_vec_string(&self, data: Vec<String>) -> SendResult {
        self.send_one(data.into()).await
    }
    async fn send_vec_string(&self, data: Vec<Vec<String>>) -> SendResult {
        unimplemented!()
    }
}

impl From<Input> for Output {
    fn from(value: Input) -> Self {
        let o = Output::new();
        o.add_transmission(&vec![value]);
        o
    }
}
