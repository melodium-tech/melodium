use crate::debug::{DataContent, Event, EventKind, TransmissionDebug};
use async_std::channel::{bounded, Receiver, Sender};
use async_std::sync::Mutex as AsyncMutex;
use async_trait::async_trait;
use melodium_common::descriptor::Flow;
use melodium_common::executive::{
    Input as ExecutiveInput, RecvResult, TrackId, TransmissionError, TransmissionValue, Value,
};

#[derive(Debug)]
pub struct Input {
    receiver: Receiver<TransmissionValue>,
    sender: Sender<TransmissionValue>,
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

    pub fn sender(&self) -> &Sender<TransmissionValue> {
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
                Ok(data) => data,
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
