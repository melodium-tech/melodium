use async_std::channel::bounded;
use async_std::channel::Receiver;
use async_std::channel::Sender;
use async_std::sync::Mutex as AsyncMutex;
use melodium_common::executive::{RecvResult, TransmissionError};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct RecvTransmitter<T> {
    receiver: Receiver<Vec<T>>,
    sender: Sender<Vec<T>>,
    buffer: AsyncMutex<Option<VecDeque<T>>>,
}

impl<T: Clone> RecvTransmitter<T> {
    pub fn new() -> Self {
        let (sender, receiver) = bounded(1);

        Self {
            sender,
            receiver,
            buffer: AsyncMutex::new(None),
        }
    }

    pub async fn receive_one(&self) -> RecvResult<T> {
        match self.receive().await {
            Ok(_) => Ok(self
                .buffer
                .lock()
                .await
                .as_mut()
                .unwrap()
                .pop_front()
                .unwrap()),
            Err(e) => Err(e),
        }
    }

    pub async fn receive_multiple(&self) -> RecvResult<Vec<T>> {
        match self.receive().await {
            Ok(_) => {
                let vec = Vec::from(self.buffer.lock().await.take().unwrap());
                Ok(vec)
            }
            Err(e) => Err(e),
        }
    }

    async fn receive(&self) -> RecvResult<()> {
        let mut buffer = self.buffer.lock().await;
        if buffer.is_none() || buffer.as_ref().unwrap().is_empty() {
            match self.receiver.recv().await {
                Ok(v) => {
                    *buffer = Some(VecDeque::from(v));
                    Ok(())
                }
                Err(_) => Err(TransmissionError::EverythingClosed),
            }
        } else {
            Ok(())
        }
    }

    pub fn close(&self) {
        self.receiver.close();
    }

    pub fn get_sender(&self) -> Sender<Vec<T>> {
        self.sender.clone()
    }
}
