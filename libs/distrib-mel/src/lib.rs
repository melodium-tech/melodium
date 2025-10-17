#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "mock", allow(unused))]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

use async_std::channel::{unbounded, Receiver, Sender};
use async_std::io::{Read, Write};
#[cfg(feature = "real")]
use async_std::net::{SocketAddr, TcpStream};
use async_std::sync::{
    Arc as AsyncArc, Barrier as AsyncBarrier, Mutex as AsyncMutex, RwLock as AsyncRwLock,
};
use common::descriptor::{Entry, Treatment};
use common::descriptor::{Identifier, Version};
use core::str::FromStr;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use futures::{pin_mut, select, FutureExt};
#[cfg(feature = "real")]
use futures_rustls::client::TlsStream;
use melodium_core::*;
#[cfg(feature = "real")]
use melodium_distribution::{
    AskDistribution, CloseInput, CloseOutput, InputData, Instanciate, InstanciateStatus,
    LoadAndLaunch, Message, Protocol,
};
use melodium_macro::{mel_model, mel_package, mel_treatment};
use melodium_share::{Collection, RawValue};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use std_mel::data::map::*;
use work_mel::access::*;

#[derive(Debug)]
struct Track {
    pub instancied: AtomicBool,
    pub instanciation_barrier: AsyncArc<AsyncBarrier>,
    pub instanciation_barrier_validated: AsyncArc<AtomicBool>,
    pub inputs_senders: HashMap<String, Sender<Vec<RawValue>>>,
    pub inputs_receivers: HashMap<String, Receiver<Vec<RawValue>>>,
    pub outputs_senders: HashMap<String, Sender<Vec<RawValue>>>,
    pub outputs_receivers: HashMap<String, Receiver<Vec<RawValue>>>,
    pub io_barrier: AsyncBarrier,
}

#[cfg(feature = "real")]
#[derive(Debug)]
enum NetworkStream {
    TlsStream(TlsStream<TcpStream>),
    TcpStream(TcpStream),
}

#[cfg(feature = "real")]
impl Read for NetworkStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match &mut *self {
            NetworkStream::TlsStream(tls_stream) => std::pin::pin!(tls_stream).poll_read(cx, buf),
            NetworkStream::TcpStream(tcp_stream) => std::pin::pin!(tcp_stream).poll_read(cx, buf),
        }
    }
}

#[cfg(feature = "real")]
impl Write for NetworkStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match &mut *self {
            NetworkStream::TlsStream(tls_stream) => std::pin::pin!(tls_stream).poll_write(cx, buf),
            NetworkStream::TcpStream(tcp_stream) => std::pin::pin!(tcp_stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            NetworkStream::TlsStream(tls_stream) => std::pin::pin!(tls_stream).poll_flush(cx),
            NetworkStream::TcpStream(tcp_stream) => std::pin::pin!(tcp_stream).poll_flush(cx),
        }
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            NetworkStream::TlsStream(tls_stream) => std::pin::pin!(tls_stream).poll_close(cx),
            NetworkStream::TcpStream(tcp_stream) => std::pin::pin!(tcp_stream).poll_close(cx),
        }
    }
}

#[derive(Debug)]
#[mel_model(
    param treatment string none
    param version string none
    continuous (continuous)
    shutdown shutdown
)]
pub struct DistributionEngine {
    model: Weak<DistributionEngineModel>,
    //protocol: AsyncRwLock<Option<AsyncArc<Protocol<TlsStream<TcpStream>>>>>,
    #[cfg(feature = "real")]
    protocol: AsyncRwLock<Option<AsyncArc<Protocol<NetworkStream>>>>,
    treatment: AsyncRwLock<Option<Arc<dyn Treatment>>>,
    tracks: AsyncRwLock<HashMap<u64, AsyncArc<AsyncRwLock<Track>>>>,
    protocol_barrier: AsyncMutex<(bool, Option<AsyncArc<AsyncBarrier>>)>,
    started_once: AtomicBool,
}

impl DistributionEngine {
    fn new(model: Weak<DistributionEngineModel>) -> Self {
        Self {
            model,
            #[cfg(feature = "real")]
            protocol: AsyncRwLock::new(None),
            treatment: AsyncRwLock::new(None),
            tracks: AsyncRwLock::new(HashMap::new()),
            protocol_barrier: AsyncMutex::new((false, Some(AsyncArc::new(AsyncBarrier::new(2))))),
            started_once: AtomicBool::new(false),
        }
    }
}

#[cfg(feature = "real")]
impl DistributionEngine {
    pub async fn protocol_barrier(&self) {
        let barrier = {
            let mut lock = self.protocol_barrier.lock().await;
            if lock.0 {
                lock.1.take()
            } else {
                lock.0 = true;
                lock.1.as_ref().map(|barrier| AsyncArc::clone(barrier))
            }
        };
        if let Some(barrier) = barrier {
            barrier.wait().await;
        }
    }

    pub async fn fuse(&self) {
        if self.started_once.load(Ordering::Relaxed) {
            self.protocol_barrier().await;
        }
    }

    pub async fn start(
        &self,
        access: &work_mel::api::CommonAccess,
        params: HashMap<String, Value>,
    ) -> Result<(), String> {
        if self.started_once.swap(true, Ordering::Relaxed) {
            return Ok(());
        }

        let model = self.model.upgrade().unwrap();

        let entrypoint = match Identifier::from_str(&model.get_treatment()) {
            Ok(id) => match Version::from_str(&model.get_version()) {
                Ok(version) => id.with_version(&version),
                Err(err) => {
                    return Err(format!("'{err}' is not a valid version"));
                }
            },
            Err(err) => {
                return Err(format!("'{err}' is not a valid identifier"));
            }
        };

        let mut protocol_lock = self.protocol.write().await;

        if protocol_lock.is_none() {
            let mut protocol = None;
            let mut error_message = None;

            for ipaddr in access.addresses.iter() {
                let addrs = SocketAddr::new(*ipaddr, access.port);

                match TcpStream::connect(&addrs).await {
                    Ok(stream) => {
                        if access.disable_tls {
                            protocol = Some(Protocol::new(NetworkStream::TcpStream(stream)));
                            break;
                        } else {
                            match tls_stream(*ipaddr, stream).await {
                                Ok(prot) => {
                                    protocol = Some(prot);
                                    break;
                                }
                                Err(err) => {
                                    error_message = Some(format!("{err}"));
                                    continue;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        error_message = Some(format!("{err}"));
                        continue;
                    }
                };
            }

            if let Some(protocol) = protocol {
                match protocol
                    .send_message(Message::AskDistribution(AskDistribution {
                        melodium_version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
                        distribution_version: melodium_distribution::VERSION.clone(),
                        key: access.remote_key,
                    }))
                    .await
                {
                    Ok(_) => {
                        match protocol.recv_message().await {
                            Ok(Message::ConfirmDistribution(confirm)) => {
                                if !confirm.accept {
                                    return Err(format!("Cannot distribute, remote engine version is {} with protocol version {}, while local engine version is {} with protocol version {}.", confirm.melodium_version, confirm.distribution_version, env!("CARGO_PKG_VERSION"), melodium_distribution::VERSION));
                                }
                                if confirm.key != access.self_key {
                                    return Err("Cannot distribute, remote engine did not provided valid key.".to_string());
                                }
                            }
                            Ok(_) => {
                                return Err("Unexpected response message".to_string());
                            }
                            Err(err) => {
                                return Err(err.to_string());
                            }
                        }
                    }
                    Err(err) => {
                        return Err(err.to_string());
                    }
                }

                let treatment = match model.world().collection().get(&(&entrypoint).into()) {
                    Some(Entry::Treatment(treatment)) => Arc::clone(treatment),
                    _ => {
                        return Err("No treatment found".to_string());
                    }
                };

                *self.treatment.write().await = Some(treatment);

                let shared_collection =
                    Collection::from_entrypoint(&model.world().collection(), &entrypoint);

                match protocol
                    .send_message(Message::LoadAndLaunch(LoadAndLaunch {
                        collection: shared_collection,
                        entrypoint: (&entrypoint).into(),
                        parameters: params
                            .into_iter()
                            .map(|(name, value)| (name, value.into()))
                            .collect(),
                    }))
                    .await
                {
                    Ok(_) => match protocol.recv_message().await {
                        Ok(Message::LaunchStatus(status)) => match status {
                            melodium_distribution::LaunchStatus::Ok => {
                                *protocol_lock = Some(AsyncArc::new(protocol));
                                self.protocol_barrier().await;
                                Ok(())
                            }
                            melodium_distribution::LaunchStatus::Failure(err) => {
                                return Err(err.to_string());
                            }
                            _ => {
                                return Err("Unexpected response message".to_string());
                            }
                        },
                        Ok(_) => {
                            return Err("Unexpected response message".to_string());
                        }
                        Err(err) => {
                            return Err(err.to_string());
                        }
                    },
                    Err(err) => {
                        return Err(err.to_string());
                    }
                }
            } else if let Some(err) = error_message {
                Err(err)
            } else {
                Err("No IP address provided".to_string())
            }
        } else {
            Ok(())
        }
    }

    pub async fn stop(&self) {
        if let Some(protocol) = self.protocol.read().await.as_ref() {
            let _ = protocol.send_message(Message::Ended).await;
            protocol.close().await;
        } else if !self.started_once.load(Ordering::Relaxed) {
            self.protocol_barrier().await;
        } else {
            self.fuse().await;
        }
    }

    pub async fn distribute(&self) -> Option<(u64, AsyncArc<AsyncBarrier>, AsyncArc<AtomicBool>)> {
        if let Some(protocol) = self.protocol.read().await.as_ref() {
            let mut tracks = self.tracks.write().await;

            let id = *tracks.keys().max().unwrap_or(&0) + 1;

            if let Some(treatment) = self.treatment.read().await.as_ref() {
                let instanciation_barrier = AsyncArc::new(AsyncBarrier::new(2));
                let instanciation_barrier_validated = AsyncArc::new(false.into());

                let mut inputs_senders = HashMap::new();
                let mut inputs_receivers = HashMap::new();
                let mut outputs_senders = HashMap::new();
                let mut outputs_receivers = HashMap::new();

                let mut io = 0;
                for (name, _) in treatment.inputs() {
                    let (sender, receiver) = unbounded();
                    inputs_senders.insert(name.clone(), sender);
                    inputs_receivers.insert(name.clone(), receiver);
                    io += 1;
                }

                for (name, _) in treatment.outputs() {
                    let (sender, receiver) = unbounded();
                    outputs_senders.insert(name.clone(), sender);
                    outputs_receivers.insert(name.clone(), receiver);
                    io += 1;
                }

                let track = Track {
                    instancied: false.into(),
                    instanciation_barrier: AsyncArc::clone(&instanciation_barrier),
                    instanciation_barrier_validated: AsyncArc::clone(
                        &instanciation_barrier_validated,
                    ),
                    inputs_senders,
                    inputs_receivers,
                    outputs_senders,
                    outputs_receivers,
                    io_barrier: AsyncBarrier::new(io),
                };

                tracks.insert(id, AsyncArc::new(AsyncRwLock::new(track)));

                if protocol
                    .send_message(Message::Instanciate(Instanciate { id: id }))
                    .await
                    .is_ok()
                {
                    Some((id, instanciation_barrier, instanciation_barrier_validated))
                } else {
                    tracks.remove(&id);
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn is_ok(&self, distribution_id: &u64) -> bool {
        let track = self.tracks.read().await.get(&distribution_id).cloned();
        if let Some(track) = track {
            track.read().await.instancied.load(Ordering::Relaxed)
        } else {
            false
        }
    }

    pub async fn get_input(
        &self,
        distribution_id: &u64,
        name: &String,
    ) -> Option<Sender<Vec<RawValue>>> {
        let track = self.tracks.read().await.get(&distribution_id).cloned();
        if let Some(track) = track {
            track.read().await.io_barrier.wait().await;
            track.write().await.inputs_senders.remove(name)
        } else {
            return None;
        }
    }

    pub async fn get_output(
        &self,
        distribution_id: &u64,
        name: &String,
    ) -> Option<Receiver<Vec<RawValue>>> {
        let track = self.tracks.read().await.get(&distribution_id).cloned();
        if let Some(track) = track {
            track.read().await.io_barrier.wait().await;
            track.write().await.outputs_receivers.remove(name)
        } else {
            return None;
        }
    }

    pub async fn send_data(&self, distribution_id: &u64, name: &String) -> Result<(), ()> {
        let track = self.tracks.read().await.get(&distribution_id).cloned();
        if let Some(track) = track {
            if let Some(data_recv) = track.read().await.inputs_receivers.get(name) {
                while let Ok(data) = data_recv.try_recv() {
                    if let Some(protocol) = self.protocol.read().await.as_ref() {
                        if let Err(_) = protocol
                            .send_message(Message::InputData(InputData {
                                id: *distribution_id,
                                name: name.clone(),
                                data: data.into(),
                            }))
                            .await
                        {
                            return Err(());
                        }
                    } else {
                        return Err(());
                    }
                }
                return Ok(());
            } else {
                return Err(());
            }
        } else {
            return Err(());
        }
    }

    pub async fn close_input(&self, distribution_id: &u64, name: &String) {
        if let Some(protocol) = self.protocol.read().await.as_ref() {
            let _ = protocol
                .send_message(Message::CloseInput(CloseInput {
                    id: *distribution_id,
                    name: name.clone(),
                }))
                .await;
        }
    }

    async fn continuous(&self) {
        self.protocol_barrier().await;

        let exec = async {
            if let Some(protocol) = self.protocol.read().await.as_ref() {
                loop {
                    let msg = protocol.recv_message().await;
                    match msg {
                        Ok(Message::InstanciateStatus(instanciate_status)) => {
                            match instanciate_status {
                                InstanciateStatus::Ok { id } => {
                                    let track = self.tracks.read().await.get(&id).cloned();
                                    if let Some(track) = track {
                                        let track = track.read().await;
                                        track.instancied.store(true, Ordering::Relaxed);
                                        track.instanciation_barrier.wait().await;
                                    }
                                }
                                InstanciateStatus::Failure { id, message: _ } => {
                                    let track = self.tracks.read().await.get(&id).cloned();
                                    if let Some(track) = track {
                                        let track = track.read().await;
                                        track.instanciation_barrier.wait().await;
                                    }
                                }
                            }
                        }
                        Ok(Message::CloseInput(close_input)) => {
                            let track = self.tracks.read().await.get(&close_input.id).cloned();
                            if let Some(track) = track {
                                if let Some(input) =
                                    track.read().await.inputs_receivers.get(&close_input.name)
                                {
                                    input.close();
                                }
                            }
                        }
                        Ok(Message::OutputData(output_data)) => {
                            let track = self.tracks.read().await.get(&output_data.id).cloned();
                            if let Some(track) = track {
                                if let Some(output) =
                                    track.read().await.outputs_senders.get(&output_data.name)
                                {
                                    if output.send(output_data.data).await.is_err() {
                                        let _ = protocol
                                            .send_message(Message::CloseOutput(CloseOutput {
                                                id: output_data.id,
                                                name: output_data.name.clone(),
                                            }))
                                            .await;
                                    }
                                }
                            }
                        }
                        Ok(Message::CloseOutput(close_output)) => {
                            let track = self.tracks.read().await.get(&close_output.id).cloned();
                            if let Some(track) = track {
                                if let Some(output) =
                                    track.read().await.outputs_senders.get(&close_output.name)
                                {
                                    output.close();
                                }
                            }
                        }
                        Ok(Message::Ended) => {
                            self.close_all().await;
                            break;
                        }
                        Ok(Message::Probe) => {}
                        Ok(_) => {}
                        Err(_) => {
                            self.close_all().await;
                            break;
                        }
                    }
                }
                protocol.close().await;
            }
        }
        .fuse();

        let probe = async {
            if let Some(protocol) = self.protocol.read().await.as_ref() {
                loop {
                    async_std::task::sleep(Duration::from_secs(10)).await;
                    if protocol.send_message(Message::Probe).await.is_err() {
                        break;
                    }
                }
                protocol.close().await;
            }
        }
        .fuse();

        pin_mut!(exec, probe);

        loop {
            select! {
                () = exec => { break }
                () = probe => { break }
                complete => break,
            }
        }

        self.close_all().await;
    }

    async fn close_all(&self) {
        for (_, track) in self.tracks.read().await.iter() {
            let track = track.read().await;
            track.inputs_receivers.iter().for_each(|(_, recv)| {
                recv.close();
            });
            track.outputs_senders.iter().for_each(|(_, send)| {
                send.close();
            });
            if !track
                .instanciation_barrier_validated
                .load(Ordering::Relaxed)
            {
                track.instanciation_barrier.wait().await;
                track
                    .instanciation_barrier_validated
                    .store(true, Ordering::Relaxed);
            }
        }
    }

    fn shutdown(&self) {
        async_std::task::block_on(async move {
            self.close_all().await;
            if let Some(protocol) = (*self.protocol.read().await).as_ref().cloned() {
                let _ = protocol.send_message(Message::Ended).await;
            }
        });
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}

#[cfg(feature = "mock")]
impl DistributionEngine {
    pub async fn continuous(&self) {}

    fn shutdown(&self) {}
    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}

#[mel_treatment(
    model distributor DistributionEngine
    input access Block<Access>
    output ready Block<void>
    output failed Block<void>
    output error Block<string>
)]
pub async fn start(params: Map) {
    let model = DistributionEngineModel::into(distributor);
    let distributor = model.inner();

    let params = params.map.clone();

    #[cfg(feature = "real")]
    if let Ok(access) = access.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Access>()
            .unwrap()
    }) {
        match distributor.start(&access.0, params).await {
            Ok(_) => {
                let _ = ready.send_one(().into()).await;
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(err.into()).await;
                distributor.fuse().await;
            }
        }
    } else {
        distributor.stop().await;
    }
    #[cfg(feature = "mock")]
    {
        let _ = failed.send_one(().into()).await;
        let _ = error.send_one("Mock mode".to_string().into()).await;
    }
}

#[mel_treatment(
    model distributor DistributionEngine
    input trigger Block<void>
)]
pub async fn stop() {
    let model = DistributionEngineModel::into(distributor);
    let distributor = model.inner();

    #[cfg(feature = "real")]
    if let Ok(_) = trigger.recv_one().await {
        distributor.stop().await;
    }
}

#[mel_treatment(
    model distributor DistributionEngine
    input trigger Block<void>
    output distribution_id Block<u64>
    output failed Block<void>
    output error Block<string>
)]
pub async fn distribute() {
    let model = DistributionEngineModel::into(distributor);
    let distributor = model.inner();

    #[cfg(feature = "real")]
    if let Ok(_) = trigger.recv_one().await {
        if let Some((id, barrier, validation)) = distributor.distribute().await {
            if !validation.load(Ordering::Relaxed) {
                barrier.wait().await;
                validation.store(true, Ordering::Relaxed);
                if distributor.is_ok(&id).await {
                    let _ = distribution_id.send_one(id.into()).await;
                } else {
                    let _ = failed.send_one(().into()).await;
                    let _ = error
                        .send_one("Instanciation failed".to_string().into())
                        .await;
                }
            }
        } else {
            let _ = failed.send_one(().into()).await;
            let _ = error
                .send_one("Distribution failed".to_string().into())
                .await;
        }
    }
    #[cfg(feature = "mock")]
    {
        let _ = failed.send_one(().into()).await;
        let _ = error.send_one("Mock mode".to_string().into()).await;
    }
}

#[mel_treatment(
    model distributor DistributionEngine
    generic D (Deserialize)
    input distribution_id Block<u64>
    output data Stream<D>
)]
pub async fn recv_stream(name: string) {
    let datatype = D;

    #[cfg(feature = "real")]
    if let Ok(distribution_id) = distribution_id
        .recv_one()
        .await
        .map(|val| GetData::<u64>::try_data(val).unwrap())
    {
        let model = DistributionEngineModel::into(distributor);
        let distributor = model.inner();
        let collection = distributor.model.upgrade().unwrap().world().collection();

        if let Some(receiver) = distributor.get_output(&distribution_id, &name).await {
            while let Ok(recv_data) = receiver.recv().await {
                let recv_data: Vec<_> = recv_data
                    .into_iter()
                    .map(|v| v.to_value(&collection))
                    .collect();
                if recv_data
                    .iter()
                    .any(|d| d.as_ref().map(|v| v.datatype() != datatype).unwrap_or(true))
                {
                    receiver.close();
                    break;
                }

                let recv_data = recv_data.into_iter().map(|v| v.unwrap()).collect();

                if data
                    .send_many(TransmissionValue::Other(recv_data))
                    .await
                    .is_err()
                {
                    receiver.close();
                }
            }
        }
    }
}

#[mel_treatment(
    model distributor DistributionEngine
    generic D (Deserialize)
    input distribution_id Block<u64>
    output data Block<D>
)]
pub async fn recv_block(name: string) {
    let datatype = D;

    #[cfg(feature = "real")]
    if let Ok(distribution_id) = distribution_id
        .recv_one()
        .await
        .map(|val| GetData::<u64>::try_data(val).unwrap())
    {
        let model = DistributionEngineModel::into(distributor);
        let distributor = model.inner();
        let collection = distributor.model.upgrade().unwrap().world().collection();

        if let Some(receiver) = distributor.get_output(&distribution_id, &name).await {
            while let Ok(recv_data) = receiver.recv().await {
                if let Some(value) = recv_data.first() {
                    if let Some(value) = value.to_value(&collection) {
                        if value.datatype() == datatype {
                            let _ = data.send_one(value).await;
                        }
                    }
                    receiver.close();
                }
            }
        }
    }
}

#[mel_treatment(
    model distributor DistributionEngine
    generic S (Serialize)
    input distribution_id Block<u64>
    input data Stream<S>
)]
pub async fn send_stream(name: string) {
    #[cfg(feature = "real")]
    if let Ok(distribution_id) = distribution_id
        .recv_one()
        .await
        .map(|val| GetData::<u64>::try_data(val).unwrap())
    {
        let model = DistributionEngineModel::into(distributor);
        let distributor = model.inner();

        if let Some(sender) = distributor.get_input(&distribution_id, &name).await {
            let mut voluntary_close = true;
            while let Ok(data) = data
                .recv_many()
                .await
                .map(|values| TryInto::<Vec<Value>>::try_into(values).unwrap())
            {
                if sender
                    .send(data.into_iter().map(|v| v.into()).collect())
                    .await
                    .is_err()
                {
                    voluntary_close = false;
                    break;
                }

                if distributor
                    .send_data(&distribution_id, &name)
                    .await
                    .is_err()
                {
                    voluntary_close = false;
                    break;
                }
            }

            if voluntary_close {
                distributor.close_input(&distribution_id, &name).await;
            }
        }
    }
}

#[mel_treatment(
    model distributor DistributionEngine
    generic S (Serialize)
    input distribution_id Block<u64>
    input data Block<S>
)]
pub async fn send_block(name: string) {
    #[cfg(feature = "real")]
    if let Ok(distribution_id) = distribution_id
        .recv_one()
        .await
        .map(|val| GetData::<u64>::try_data(val).unwrap())
    {
        let model = DistributionEngineModel::into(distributor);
        let distributor = model.inner();

        if let Some(sender) = distributor.get_input(&distribution_id, &name).await {
            let mut voluntary_close = true;
            if let Ok(data) = data.recv_one().await {
                if sender.send(vec![data.into()]).await.is_err() {
                    voluntary_close = false;
                } else {
                    if distributor
                        .send_data(&distribution_id, &name)
                        .await
                        .is_err()
                    {
                        voluntary_close = false;
                    }
                }
            }
            if voluntary_close {
                distributor.close_input(&distribution_id, &name).await;
            }
        }
    }
}

#[cfg(feature = "real")]
async fn tls_stream(
    ip: std::net::IpAddr,
    stream: TcpStream,
) -> std::io::Result<Protocol<NetworkStream>> {
    use futures_rustls::rustls::{
        pki_types::ServerName, version::TLS13, ClientConfig, RootCertStore,
    };
    use futures_rustls::TlsConnector;

    let mut root_store = RootCertStore::empty();
    root_store.add_parsable_certificates(
        rustls_pemfile::certs(&mut melodium_certs::ROOT_CERTIFICATE.as_slice())
            .filter_map(|cert| cert.ok()),
    );
    let config = ClientConfig::builder_with_protocol_versions(&[&TLS13])
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(std::sync::Arc::new(config));
    Ok(Protocol::new(NetworkStream::TlsStream(
        connector
            .connect(ServerName::IpAddress(ip.into()), stream)
            .await?,
    )))
}

mel_package!();
