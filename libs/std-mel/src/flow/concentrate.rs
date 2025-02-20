use async_std::{
    channel::{bounded, Receiver, Sender, WeakSender},
    sync::Mutex,
};
use melodium_core::{
    common::{descriptor::DataType, executive::TrackId},
    *,
};
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::{hash_map::Entry as HashMapEntry, HashMap};

#[derive(Debug)]
enum SenderState {
    Strong(Sender<Value>),
    Weak(WeakSender<Value>),
}

#[derive(Debug)]
struct TrackEntry {
    pub track_sender: Mutex<SenderState>,
    pub track_receiver: Mutex<Option<Receiver<Value>>>,
}

#[mel_model]
#[derive(Debug)]
pub struct Concentrator {
    _model: std::sync::Weak<ConcentratorModel>,
    tracks: Mutex<HashMap<TrackId, Vec<(DataType, TrackEntry)>>>,
}

impl Concentrator {
    pub fn new(model: std::sync::Weak<ConcentratorModel>) -> Self {
        Self {
            _model: model,
            tracks: Mutex::new(HashMap::new()),
        }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    pub async fn track_sender(
        &self,
        track_id: TrackId,
        data_type: DataType,
    ) -> Option<Sender<Value>> {
        match self.tracks.lock().await.entry(track_id) {
            HashMapEntry::Occupied(mut occupied_entry) => {
                let entries = occupied_entry.get_mut();

                if let Some((_, entry)) = entries.iter_mut().find(|(dt, _)| dt == &data_type) {
                    let mut weak_sender = None;
                    let sender = match &entry.track_sender.get_mut() {
                        SenderState::Strong(sender) => {
                            weak_sender = Some(sender.downgrade());
                            Some(sender.clone())
                        }
                        SenderState::Weak(weak_sender) => weak_sender.upgrade(),
                    };

                    if let Some(weak_sender) = weak_sender {
                        *entry.track_sender.get_mut() = SenderState::Weak(weak_sender);
                    }

                    sender
                } else {
                    let couple = bounded(500);
                    let track_entry = TrackEntry {
                        track_sender: Mutex::new(SenderState::Weak(couple.0.downgrade())),
                        track_receiver: Mutex::new(Some(couple.1)),
                    };

                    entries.push((data_type, track_entry));

                    Some(couple.0)
                }
            }
            HashMapEntry::Vacant(vacant_entry) => {
                let couple = bounded(500);
                let track_entry = TrackEntry {
                    track_sender: Mutex::new(SenderState::Weak(couple.0.downgrade())),
                    track_receiver: Mutex::new(Some(couple.1)),
                };

                let entries = vec![(data_type, track_entry)];
                vacant_entry.insert(entries);

                Some(couple.0)
            }
        }
    }

    pub async fn track_receiver(
        &self,
        track_id: TrackId,
        data_type: DataType,
    ) -> Option<Receiver<Value>> {
        match self.tracks.lock().await.entry(track_id) {
            HashMapEntry::Occupied(mut occupied_entry) => {
                let entries = occupied_entry.get_mut();

                if let Some((_, entry)) = entries.iter_mut().find(|(dt, _)| dt == &data_type) {
                    entry.track_receiver.get_mut().take()
                } else {
                    let couple = bounded(500);
                    let track_entry = TrackEntry {
                        track_sender: Mutex::new(SenderState::Strong(couple.0)),
                        track_receiver: Mutex::new(None),
                    };

                    entries.push((data_type, track_entry));

                    Some(couple.1)
                }
            }
            HashMapEntry::Vacant(vacant_entry) => {
                let couple = bounded(500);
                let track_entry = TrackEntry {
                    track_sender: Mutex::new(SenderState::Strong(couple.0)),
                    track_receiver: Mutex::new(None),
                };

                let entries = vec![(data_type, track_entry)];
                vacant_entry.insert(entries);

                Some(couple.1)
            }
        }
    }
}

#[mel_treatment(
    model concentrator Concentrator
    generic T ()
    input data Stream<T>
)]
pub async fn concentrate() {
    let model = ConcentratorModel::into(concentrator);
    let concentrator = model.inner();

    let data_type = T;

    if let Some(sender) = concentrator.track_sender(track_id, data_type).await {
        while let Ok(value) = data.recv_one().await {
            check!(sender.send(value).await)
        }
    }
}

#[mel_treatment(
    model concentrator Concentrator
    generic T ()
    input data Block<T>
)]
pub async fn concentrateBlock() {
    let model = ConcentratorModel::into(concentrator);
    let concentrator = model.inner();

    let data_type = T;

    if let Some(sender) = concentrator.track_sender(track_id, data_type).await {
        if let Ok(value) = data.recv_one().await {
            let _ = sender.send(value).await;
        }
    }
}

#[mel_treatment(
    model concentrator Concentrator
    generic T ()
    input trigger Block<T>
    output data Stream<T>
)]
pub async fn concentrated() {
    let model = ConcentratorModel::into(concentrator);
    let concentrator = model.inner();

    let data_type = T;

    if let Ok(_) = trigger.recv_one().await {
        if let Some(receiver) = concentrator.track_receiver(track_id, data_type).await {
            while let Ok(value) = receiver.recv().await {
                check!(data.send_one(value).await)
            }
        }
    }
}
