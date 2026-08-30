use crate::debug::{DataContent, Event, EventKind, TransmissionDebug};
use crate::transmission::own;
use async_std::channel::{bounded, Receiver, Sender};
use async_std::sync::Mutex as AsyncMutex;
use async_trait::async_trait;
use melodium_common::descriptor::Flow;
use melodium_common::executive::{
    Input as ExecutiveInput, RecvResult, TrackId, TransmissionError, TransmissionValue, Value,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Input {
    // Carries `Arc<TransmissionValue>` rather than an owned batch so a fan-out `Output`
    // can hand every receiver a cheap refcount bump instead of a deep clone of the whole
    // batch. Ownership is reclaimed lazily on receive (see `own`): whichever `Input`
    // happens to be the last to read a given batch gets it for free via `Arc::try_unwrap`.
    receiver: Receiver<Arc<TransmissionValue>>,
    sender: Sender<Arc<TransmissionValue>>,
    buffer: AsyncMutex<Option<TransmissionValue>>,
    flow: Flow,
    track_id: TrackId,
    debug: TransmissionDebug,
}

impl Input {
    pub fn new(flow: Flow, track_id: TrackId, debug: TransmissionDebug) -> Self {
        let (sender, receiver) = bounded(1);
        Self {
            receiver,
            sender,
            buffer: AsyncMutex::new(None),
            flow,
            track_id,
            debug,
        }
    }

    pub fn sender(&self) -> &Sender<Arc<TransmissionValue>> {
        &self.sender
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
}

#[async_trait]
impl ExecutiveInput for Input {
    fn close(&self) {
        self.receiver.close();
        match &self.debug {
            TransmissionDebug::None => {}
            TransmissionDebug::Basic(world, transmission_details)
            | TransmissionDebug::Detailed(world, transmission_details) => {
                world.send_debug(Event::new(EventKind::InputClosed {
                    input: transmission_details.clone(),
                    track_id: self.track_id.clone(),
                }))
            }
        }
    }

    async fn recv_many(&self) -> RecvResult<TransmissionValue> {
        let mut lock = self.buffer.lock().await;
        let data = if let Some(data) = lock.take() {
            data
        } else {
            match self.receiver.recv().await {
                Ok(data) => own(data),
                Err(_) => return Err(TransmissionError::EverythingClosed),
            }
        };

        match &self.debug {
            TransmissionDebug::None => {}
            TransmissionDebug::Basic(world, transmission_details)
            | TransmissionDebug::Detailed(world, transmission_details) => {
                world
                    .send_debug_async(Event::new(EventKind::DataReceived {
                        input: transmission_details.clone(),
                        track_id: self.track_id.clone(),
                        data: DataContent::Count { count: data.len() },
                    }))
                    .await
            }
        }

        Ok(data)
    }

    async fn recv_one(&self) -> RecvResult<Value> {
        let mut lock = self.buffer.lock().await;
        let value = if let Some(data) = lock.as_mut() {
            data.pop_front().ok_or(TransmissionError::NoData)
        } else {
            match self.receiver.recv().await {
                Ok(data) => {
                    let mut data = own(data);
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

        match &self.debug {
            TransmissionDebug::None => {}
            TransmissionDebug::Basic(world, transmission_details)
            | TransmissionDebug::Detailed(world, transmission_details) => {
                world
                    .send_debug_async(Event::new(EventKind::DataReceived {
                        input: transmission_details.clone(),
                        track_id: self.track_id.clone(),
                        data: DataContent::Count { count: 1 },
                    }))
                    .await
            }
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
            flow: self.flow.clone(),
            track_id: self.track_id,
            debug: self.debug.clone(),
        }
    }
}
