#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
use async_native_tls::TlsStream;
use async_std::channel::{unbounded, Receiver, Sender};
use async_std::io::{Read, Write};
use async_std::net::{SocketAddr, TcpStream};
use async_std::sync::{Arc as AsyncArc, Barrier as AsyncBarrier, RwLock as AsyncRwLock};
use common::descriptor::{Entry, Treatment};
use common::descriptor::{Identifier, Version};
use core::str::FromStr;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use futures::{pin_mut, select, FutureExt};
#[cfg(any(
    all(not(target_os = "windows"), not(target_vendor = "apple")),
    all(target_os = "windows", target_env = "gnu")
))]
use futures_rustls::client::TlsStream;
use melodium_core::*;
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
use std_mel::data::*;
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

#[derive(Debug)]
#[mel_model(
    param treatment string none
    param version string none
    continuous (continuous)
    shutdown shutdown
)]
pub struct DistributionEngine {
    model: Weak<DistributionEngineModel>,
    protocol: AsyncRwLock<Option<AsyncArc<Protocol<TlsStream<TcpStream>>>>>,
    treatment: AsyncRwLock<Option<Arc<dyn Treatment>>>,
    tracks: AsyncRwLock<HashMap<u64, Track>>,
    protocol_barrier: AsyncBarrier,
    started_once: AtomicBool,
}

impl DistributionEngine {
    fn new(model: Weak<DistributionEngineModel>) -> Self {
        Self {
            model,
            protocol: AsyncRwLock::new(None),
            treatment: AsyncRwLock::new(None),
            tracks: AsyncRwLock::new(HashMap::new()),
            protocol_barrier: AsyncBarrier::new(2),
            started_once: AtomicBool::new(false),
        }
    }

    pub async fn fuse(&self) {
        if self.started_once.load(Ordering::Relaxed) {
            self.protocol_barrier.wait().await;
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

                match TcpStream::connect(addrs).await {
                    Ok(stream) => match tls_stream(*ipaddr, stream).await {
                        Ok(prot) => {
                            protocol = Some(prot);
                            break;
                        }
                        Err(err) => {
                            error_message = Some(err.to_string());
                            continue;
                        }
                    },
                    Err(err) => {
                        error_message = Some(err.to_string());
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
                                self.protocol_barrier.wait().await;
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
        } else if !self.started_once.load(Ordering::Relaxed) {
            self.protocol_barrier.wait().await;
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

                tracks.insert(id, track);

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
        self.tracks
            .read()
            .await
            .get(&distribution_id)
            .map(|track| track.instancied.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    pub async fn get_input(
        &self,
        distribution_id: &u64,
        name: &String,
    ) -> Option<Sender<Vec<RawValue>>> {
        if let Some(track) = self.tracks.read().await.get(&distribution_id) {
            track.io_barrier.wait().await;
        } else {
            return None;
        }
        self.tracks
            .write()
            .await
            .get_mut(&distribution_id)
            .map(|track| track.inputs_senders.remove(name))
            .flatten()
    }

    pub async fn get_output(
        &self,
        distribution_id: &u64,
        name: &String,
    ) -> Option<Receiver<Vec<RawValue>>> {
        if let Some(track) = self.tracks.read().await.get(&distribution_id) {
            track.io_barrier.wait().await;
        } else {
            return None;
        }
        self.tracks
            .write()
            .await
            .get_mut(&distribution_id)
            .map(|track| track.outputs_receivers.remove(name))
            .flatten()
    }

    pub async fn send_data(&self, distribution_id: &u64, name: &String) {
        if let Some(data_recv) = self
            .tracks
            .read()
            .await
            .get(&distribution_id)
            .map(|track| track.inputs_receivers.get(name))
            .flatten()
        {
            while let Ok(data) = data_recv.try_recv() {
                if let Some(protocol) = self.protocol.read().await.as_ref() {
                    let _ = protocol
                        .send_message(Message::InputData(InputData {
                            id: *distribution_id,
                            name: name.clone(),
                            data: data.into(),
                        }))
                        .await;
                }
            }
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
        eprintln!("Awaiting distribution");
        self.protocol_barrier.wait().await;

        let exec = async {
            if let Some(protocol) = self.protocol.read().await.as_ref() {
                loop {
                    match protocol.recv_message().await {
                        Ok(Message::InstanciateStatus(instanciate_status)) => {
                            match instanciate_status {
                                InstanciateStatus::Ok { id } => {
                                    if let Some(track) = self.tracks.read().await.get(&id) {
                                        track.instancied.store(true, Ordering::Relaxed);
                                        track.instanciation_barrier.wait().await;
                                    }
                                }
                                InstanciateStatus::Failure { id, message: _ } => {
                                    if let Some(track) = self.tracks.read().await.get(&id) {
                                        track.instanciation_barrier.wait().await;
                                    }
                                }
                            }
                        }
                        Ok(Message::CloseInput(close_input)) => {
                            if let Some(input) = self
                                .tracks
                                .read()
                                .await
                                .get(&close_input.id)
                                .map(|track| track.inputs_receivers.get(&close_input.name))
                                .flatten()
                            {
                                input.close();
                            }
                        }
                        Ok(Message::OutputData(output_data)) => {
                            if let Some(output) = self
                                .tracks
                                .read()
                                .await
                                .get(&output_data.id)
                                .map(|track| track.outputs_senders.get(&output_data.name))
                                .flatten()
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
                        Ok(Message::CloseOutput(close_output)) => {
                            if let Some(output) = self
                                .tracks
                                .read()
                                .await
                                .get(&close_output.id)
                                .map(|track| track.outputs_senders.get(&close_output.name))
                                .flatten()
                            {
                                output.close();
                            }
                        }
                        Ok(Message::Ended) => {
                            eprintln!("Ending distribution");
                            self.close_all().await;
                            break;
                        }
                        Ok(Message::Probe) => {}
                        Ok(_) => {}
                        Err(_) => {
                            eprintln!("Error on distribution");
                            self.close_all().await;
                            break;
                        }
                    }
                }
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

        eprintln!("Finishing distribution");
    }

    async fn close_all(&self) {
        for (_, track) in self.tracks.read().await.iter() {
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
        distributor.fuse().await;
    }
}

#[mel_treatment(
    model distributor DistributionEngine
    input trigger Block<void>
)]
pub async fn stop() {
    let model = DistributionEngineModel::into(distributor);
    let distributor = model.inner();

    if let Ok(_) = trigger.recv_one().await {
        eprintln!("Triggering distrib stop");
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
}

#[mel_treatment(
    model distributor DistributionEngine
    generic D (Deserialize)
    input distribution_id Block<u64>
    output data Stream<D>
)]
pub async fn recv_stream(name: string) {
    let datatype = D;

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
                distributor.send_data(&distribution_id, &name).await;
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
                    distributor.send_data(&distribution_id, &name).await;
                }
            }
            if voluntary_close {
                distributor.close_input(&distribution_id, &name).await;
            }
        }
    }
}

#[cfg(any(
    all(not(target_os = "windows"), not(target_vendor = "apple")),
    all(target_os = "windows", target_env = "gnu")
))]
async fn tls_stream<IO>(
    ip: std::net::IpAddr,
    stream: IO,
) -> std::io::Result<Protocol<TlsStream<IO>>>
where
    IO: Read + Write + Unpin + Send,
{
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
    Ok(Protocol::new(
        connector
            .connect(ServerName::IpAddress(ip.into()), stream)
            .await?,
    ))
}

#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
async fn tls_stream<IO>(
    ip: std::net::IpAddr,
    stream: IO,
) -> std::io::Result<Protocol<TlsStream<IO>>>
where
    IO: Read + Write + Unpin + Send,
{
    use async_native_tls::{Certificate, Protocol as NativeTlsProtocol, TlsConnector};
    use std::io::{Error, ErrorKind};

    match TlsConnector::new()
        .min_protocol_version(Some(NativeTlsProtocol::Tlsv12))
        .add_root_certificate(
            Certificate::from_pem(melodium_certs::ROOT_CERTIFICATE.as_slice())
                .map_err(|err| Error::new(ErrorKind::Other, err))?,
        )
        .connect(ip.to_string(), stream)
        .await
    {
        Ok(stream) => Ok(Protocol::new(stream)),
        Err(err) => Err(Error::new(ErrorKind::Other, err)),
    }
}

mel_package!();
