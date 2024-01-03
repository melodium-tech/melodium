use async_std::channel::{bounded, Receiver, Sender};
use async_std::sync::Mutex as AsyncMutex;
use async_trait::async_trait;
use melodium_common::executive::{
    Input as ExecutiveInput, RecvResult, TransmissionError, TransmissionValue, Value,
};

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
