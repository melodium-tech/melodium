use super::receive_transmitter::RecvTransmitter;
use async_std::channel::Sender;
use melodium_common::executive::{SendResult, TransmissionError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

const BUFFER_LIMIT: usize = 2usize.pow(20);

#[derive(Debug)]
pub struct SendTransmitter<T> {
    senders: Mutex<Vec<Sender<Vec<T>>>>,
    buffer: Mutex<Vec<T>>,

    has_receivers: AtomicBool,
}

impl<T: Clone> SendTransmitter<T> {
    pub fn new() -> Self {
        Self {
            senders: Mutex::new(Vec::new()),
            buffer: Mutex::new(Vec::with_capacity(BUFFER_LIMIT)),
            has_receivers: AtomicBool::new(false),
        }
    }

    pub fn add_transmitter(&self, transmitter: &RecvTransmitter<T>) {
        let sender = transmitter.get_sender();
        self.senders.lock().unwrap().push(sender);

        self.has_receivers.store(true, Ordering::Relaxed);
    }

    pub async fn send(&self, data: T) -> SendResult {
        if !self.has_receivers.load(Ordering::Relaxed) {
            return Err(TransmissionError::NoReceiver);
        } else {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push(data);
        }

        self.check_send().await
    }

    pub async fn send_multiple(&self, data: Vec<T>) -> SendResult {
        if !self.has_receivers.load(Ordering::Relaxed) {
            return Err(TransmissionError::NoReceiver);
        } else if data.is_empty() {
            return Ok(());
        } else {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.extend(data);
        }

        self.check_send().await
    }

    async fn check_send(&self) -> SendResult {
        let buffer_len = self.buffer.lock().unwrap().len();

        if buffer_len >= BUFFER_LIMIT {
            self.do_send().await
        } else {
            Ok(())
        }
    }

    async fn do_send(&self) -> SendResult {
        let buffer = self.buffer.lock().unwrap().clone();

        let mut statuses = Vec::new();
        let senders = self.senders.lock().unwrap().clone();
        for sender in senders.iter() {
            statuses.push(match sender.send(buffer.clone()).await {
                Ok(()) => true,
                Err(_) => false,
            });
        }

        let status = if let Some(_) = statuses.iter().find(|s| **s) {
            Ok(())
        } else {
            Err(TransmissionError::EverythingClosed)
        };

        self.buffer.lock().unwrap().clear();

        return status;
    }

    pub async fn close(&self) {
        // In closing we don't care for send result
        let _result = self.do_send().await;

        self.senders.lock().unwrap().iter().for_each(|s| {
            s.close();
        });
    }
}
