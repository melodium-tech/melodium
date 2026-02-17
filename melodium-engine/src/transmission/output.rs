use crate::debug::{DataContent, Event, EventKind};
use crate::transmission::{Input, TransmissionDebug, TransmissionDetails};
use async_std::channel::{Sender, TrySendError};
use async_std::sync::Mutex as AsyncMutex;
use async_trait::async_trait;
use core::sync::atomic::{AtomicUsize, Ordering};
use futures::stream::{FuturesUnordered, StreamExt};
use melodium_common::descriptor::Flow;
use melodium_common::executive::{
    Output as ExecutiveOutput, SendResult, TrackId, TransmissionError, TransmissionValue, Value,
};
use std::sync::{Arc, Mutex};

const LIMIT: usize = 2usize.pow(20);

#[derive(Debug)]
pub struct Output {
    senders: Mutex<Arc<Vec<(Sender<TransmissionValue>, Option<TransmissionDetails>)>>>,
    count_receivers: AtomicUsize,
    buffer: AsyncMutex<Option<TransmissionValue>>,
    flow: Flow,
    track_id: TrackId,
    debug: TransmissionDebug,
}

impl Output {
    pub fn new(flow: Flow, track_id: TrackId, debug: TransmissionDebug) -> Self {
        Self {
            senders: Mutex::new(Arc::new(Vec::new())),
            count_receivers: AtomicUsize::new(0),
            buffer: AsyncMutex::new(None),
            flow,
            track_id,
            debug,
        }
    }

    pub fn flow(&self) -> &Flow {
        &self.flow
    }

    pub fn track_id(&self) -> &TrackId {
        &self.track_id
    }

    pub fn transmission_debug(&self) -> &TransmissionDebug {
        &self.debug
    }

    pub fn add_transmission(&self, inputs: &Vec<Input>) {
        let mut senders = self.senders.lock().unwrap();
        let count = inputs.len();
        // An output is not supposed to have transmission added while it is already in use,
        // so get_mut on Arc is doable.
        if let Some(senders) = Arc::get_mut(&mut senders) {
            for input in inputs {
                senders.push((
                    input.sender().clone(),
                    match input.transmission_debug() {
                        TransmissionDebug::None => None,
                        TransmissionDebug::Basic(_, details)
                        | TransmissionDebug::Detailed(_, details) => Some(details.clone()),
                    },
                ));
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
            if self.flow == Flow::Block || buffer_len >= LIMIT || force {
                match self.count_receivers.load(Ordering::Relaxed) {
                    0 => Err(TransmissionError::NoReceiver),
                    1 => {
                        let senders = Arc::clone(&self.senders.lock().unwrap());
                        if let Some((sender, input_transmission_details)) = senders.first() {
                            match sender.send(data).await {
                                Ok(_) => {
                                    match (&self.debug, input_transmission_details) {
                                        (_, None) | (TransmissionDebug::None, _) => {}
                                        (
                                            TransmissionDebug::Basic(world, output_details),
                                            Some(input_details),
                                        )
                                        | (
                                            TransmissionDebug::Detailed(world, output_details),
                                            Some(input_details),
                                        ) => {
                                            world
                                                .send_debug_async(Event::new(
                                                    EventKind::DataTransmitted {
                                                        output: output_details.clone(),
                                                        input: input_details.clone(),
                                                        track_id: self.track_id.clone(),
                                                        data: DataContent::Count {
                                                            count: buffer_len,
                                                        },
                                                    },
                                                ))
                                                .await
                                        }
                                    }
                                    Ok(())
                                }
                                Err(_) => Err(TransmissionError::EverythingClosed),
                            }
                        } else {
                            Err(TransmissionError::NoReceiver)
                        }
                    }
                    _ => {
                        let senders = Arc::clone(&self.senders.lock().unwrap());

                        let transmissions = FuturesUnordered::new();
                        for (sender, input_transmission_details) in senders.iter() {
                            let transmission = {
                                let data = &data;
                                async move {
                                    match sender.send(data.clone()).await {
                                        Ok(_) => {
                                            match (&self.debug, input_transmission_details) {
                                                (_, None) | (TransmissionDebug::None, _) => {}
                                                (
                                                    TransmissionDebug::Basic(world, output_details),
                                                    Some(input_details),
                                                )
                                                | (
                                                    TransmissionDebug::Detailed(
                                                        world,
                                                        output_details,
                                                    ),
                                                    Some(input_details),
                                                ) => {
                                                    world
                                                        .send_debug_async(Event::new(
                                                            EventKind::DataTransmitted {
                                                                output: output_details.clone(),
                                                                input: input_details.clone(),
                                                                track_id: self.track_id.clone(),
                                                                data: DataContent::Count {
                                                                    count: buffer_len,
                                                                },
                                                            },
                                                        ))
                                                        .await
                                                }
                                            }
                                            true
                                        }
                                        Err(_) => false,
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
                    }
                }
            } else {
                match self.count_receivers.load(Ordering::Relaxed) {
                    0 => Err(TransmissionError::NoReceiver),
                    1 => {
                        let senders = Arc::clone(&self.senders.lock().unwrap());
                        if let Some((sender, input_transmission_details)) = senders.first() {
                            match sender.try_send(data) {
                                Ok(_) => {
                                    match (&self.debug, input_transmission_details) {
                                        (_, None) | (TransmissionDebug::None, _) => {}
                                        (
                                            TransmissionDebug::Basic(world, output_details),
                                            Some(input_details),
                                        )
                                        | (
                                            TransmissionDebug::Detailed(world, output_details),
                                            Some(input_details),
                                        ) => {
                                            world
                                                .send_debug_async(Event::new(
                                                    EventKind::DataTransmitted {
                                                        output: output_details.clone(),
                                                        input: input_details.clone(),
                                                        track_id: self.track_id.clone(),
                                                        data: DataContent::Count {
                                                            count: buffer_len,
                                                        },
                                                    },
                                                ))
                                                .await
                                        }
                                    }
                                    Ok(())
                                }
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

                        let all_senders_not_full =
                            !senders.iter().any(|(sender, _)| sender.is_full());

                        if all_senders_not_full {
                            let transmissions = FuturesUnordered::new();
                            for (sender, input_transmission_details) in senders.iter() {
                                let transmission = {
                                    let data = &data;
                                    async move {
                                        match sender.try_send(data.clone()) {
                                            Ok(_) => {
                                                match (&self.debug, input_transmission_details) {
                                                    (_, None) | (TransmissionDebug::None, _) => {}
                                                    (
                                                        TransmissionDebug::Basic(
                                                            world,
                                                            output_details,
                                                        ),
                                                        Some(input_details),
                                                    )
                                                    | (
                                                        TransmissionDebug::Detailed(
                                                            world,
                                                            output_details,
                                                        ),
                                                        Some(input_details),
                                                    ) => {
                                                        world
                                                            .send_debug_async(Event::new(
                                                                EventKind::DataTransmitted {
                                                                    output: output_details.clone(),
                                                                    input: input_details.clone(),
                                                                    track_id: self.track_id.clone(),
                                                                    data: DataContent::Count {
                                                                        count: buffer_len,
                                                                    },
                                                                },
                                                            ))
                                                            .await
                                                    }
                                                }
                                                true
                                            }
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
        self.senders.lock().unwrap().iter().for_each(|(s, _)| {
            s.close();
        });
        match &self.debug {
            TransmissionDebug::None => {}
            TransmissionDebug::Basic(world, transmission_details)
            | TransmissionDebug::Detailed(world, transmission_details) => {
                world
                    .send_debug_async(Event::new(EventKind::OutputClosed {
                        output: transmission_details.clone(),
                        track_id: self.track_id.clone(),
                    }))
                    .await
            }
        }
    }

    async fn send_many(&self, data: TransmissionValue) -> SendResult {
        match &self.debug {
            TransmissionDebug::None => {}
            TransmissionDebug::Basic(world, transmission_details) => {
                world
                    .send_debug_async(Event::new(EventKind::DataSent {
                        output: transmission_details.clone(),
                        track_id: self.track_id.clone(),
                        data: DataContent::Count { count: data.len() },
                    }))
                    .await
            }
            TransmissionDebug::Detailed(world, transmission_details) => {
                world
                    .send_debug_async(Event::new(EventKind::DataSent {
                        output: transmission_details.clone(),
                        track_id: self.track_id.clone(),
                        data: DataContent::Values {
                            values: data.clone().into(),
                        },
                    }))
                    .await
            }
        }

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
        match &self.debug {
            TransmissionDebug::None => {}
            TransmissionDebug::Basic(world, transmission_details) => {
                world
                    .send_debug_async(Event::new(EventKind::DataSent {
                        output: transmission_details.clone(),
                        track_id: self.track_id.clone(),
                        data: DataContent::Count { count: 1 },
                    }))
                    .await
            }
            TransmissionDebug::Detailed(world, transmission_details) => {
                world
                    .send_debug_async(Event::new(EventKind::DataSent {
                        output: transmission_details.clone(),
                        track_id: self.track_id.clone(),
                        data: DataContent::Values {
                            values: vec![data.clone()],
                        },
                    }))
                    .await
            }
        }

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

    async fn force_send(&self) {
        let _ = self.check_send(true).await;
    }
}

impl From<Input> for Output {
    fn from(value: Input) -> Self {
        let o = Output::new(*value.flow(), *value.track_id(), TransmissionDebug::None);
        o.add_transmission(&vec![value]);
        o
    }
}
