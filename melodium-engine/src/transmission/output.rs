use crate::transmission::Input;
use async_std::channel::{Sender, TrySendError};
use async_std::sync::Mutex as AsyncMutex;
use async_trait::async_trait;
use core::sync::atomic::{AtomicUsize, Ordering};
use futures::stream::{FuturesUnordered, StreamExt};
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

        if buffer_len > 0 {
            // We can unwrap the `take` because buffer_len must be > 0, so buffer have value.
            let data = self.buffer.lock().await.take().unwrap();
            if buffer_len >= LIMIT || force {
                match self.count_receivers.load(Ordering::Relaxed) {
                    0 => Err(TransmissionError::NoReceiver),
                    1 => {
                        eprintln!("Exactly 1 receiver");
                        let senders = Arc::clone(&self.senders.lock().unwrap());
                        if let Some(sender) = senders.first() {
                            match sender.send(data).await {
                                Ok(_) => {eprintln!("Sent and done");Ok(())},
                                Err(_) => {eprintln!("Receiver closed");Err(TransmissionError::EverythingClosed)},
                            }
                        } else {
                            Err(TransmissionError::NoReceiver)
                        }
                    }
                    x => {
                        eprintln!("Multiple receivers: {x}");
                        let senders = Arc::clone(&self.senders.lock().unwrap());

                        let transmissions = FuturesUnordered::new();
                        for sender in senders.iter() {
                            let transmission = {
                                let data = &data;
                                async move {
                                    eprintln!("Sending for a receiver");
                                    let res = match sender.send(data.clone()).await {
                                        Ok(_) => true,
                                        Err(_) => false,
                                    };
                                    eprintln!("Sent for a receiver ({res})");
                                    res
                                }
                            };
                            transmissions.push(transmission);
                        }

                        let statuses: Vec<_> = transmissions.collect().await;
                        eprintln!("Everything sent");

                        if let Some(_) = statuses.iter().find(|s| **s) {
                            eprintln!("Not all closed");
                            Ok(())
                        } else {
                            eprintln!("Everything closed");
                            Err(TransmissionError::EverythingClosed)
                        }
                    }
                }
            } else {
                match self.count_receivers.load(Ordering::Relaxed) {
                    0 => Err(TransmissionError::NoReceiver),
                    1 => {
                        let senders = Arc::clone(&self.senders.lock().unwrap());
                        if let Some(sender) = senders.first() {
                            match sender.try_send(data) {
                                Ok(_) => Ok(()),
                                Err(TrySendError::Full(data)) => {
                                    self.buffer.lock().await.replace(data);
                                    Ok(())
                                }
                                Err(TrySendError::Closed(_)) => {
                                    Err(TransmissionError::EverythingClosed)
                                }
                            }
                        } else {
                            Err(TransmissionError::NoReceiver)
                        }
                    }
                    _ => {
                        let senders = Arc::clone(&self.senders.lock().unwrap());

                        let all_senders_not_full = !senders.iter().any(|sender| sender.is_full());

                        if all_senders_not_full {
                            let transmissions = FuturesUnordered::new();
                            for sender in senders.iter() {
                                let transmission = {
                                    let data = &data;
                                    async move {
                                        match sender.try_send(data.clone()) {
                                            Ok(_) => true,
                                            Err(TrySendError::Full(_)) => unreachable!(),
                                            Err(TrySendError::Closed(_)) => false,
                                        }
                                    }
                                };
                                transmissions.push(transmission);
                            }

                            let statuses: Vec<_> = transmissions.collect().await;

                            if let Some(_) = statuses.iter().find(|s| **s) {
                                Ok(())
                            } else {
                                Err(TransmissionError::EverythingClosed)
                            }
                        } else {
                            self.buffer.lock().await.replace(data);
                            Ok(())
                        }
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
}

impl From<Input> for Output {
    fn from(value: Input) -> Self {
        let o = Output::new();
        o.add_transmission(&vec![value]);
        o
    }
}
