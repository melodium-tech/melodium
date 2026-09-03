#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "mock", allow(unused))]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

use async_std::channel::{bounded, Receiver, Sender};
use async_std::io::{Read, Write};
#[cfg(feature = "real")]
use async_std::net::{SocketAddr, TcpStream};
use async_std::sync::{Arc as AsyncArc, Barrier as AsyncBarrier, RwLock as AsyncRwLock};
use common::descriptor::{Entry, Treatment};
use common::descriptor::{Identifier, Version};
use core::str::FromStr;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use event_listener::{Event, IntoNotification};
#[cfg(feature = "real")]
use futures::stream::{unfold, FuturesUnordered};
#[cfg(feature = "real")]
use futures::StreamExt;
use futures::{pin_mut, select, FutureExt};
#[cfg(feature = "real")]
use futures_rustls::client::TlsStream;
use melodium_core::*;
#[cfg(feature = "real")]
use melodium_distribution::{
    chunk_raw_values, max_batch_chunk_bytes, max_concurrent_messages, AskDistribution, CloseInput,
    CloseOutput, InputData, Instanciate, InstanciateStatus, LoadAndLaunch, Message, Protocol,
};
use melodium_macro::{mel_model, mel_package, mel_treatment};
use melodium_share::{Collection, RawValue};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use std_mel::data::map::*;
use uuid::Uuid;
use work_mel::access::*;

/// Number of attempts made to send a keepalive `Probe` before treating the connection as
/// genuinely dead. See the comment at the probe retry loop for why a single failure isn't
/// trusted on its own.
#[cfg(feature = "real")]
const PROBE_RETRY_ATTEMPTS: u32 = 3;
/// Delay between probe retry attempts.
#[cfg(feature = "real")]
const PROBE_RETRY_DELAY: Duration = Duration::from_secs(2);

/// Capacity of the per-port input/output data channels backing a distributed track.
///
/// In practice each of these channels holds at most one pending batch: the sending side always
/// awaits a full drain-and-network-send right after pushing onto it (see `send_data`), and the
/// receiving side (`Message::OutputData` handling in `continuous`) awaits the channel directly,
/// so a full channel already back-pressures the connection's message-read loop. The bound here
/// exists so that stays true - a bounded channel with real, deliberate slack instead of an
/// unbounded one that could otherwise grow without limit if either assumption ever changes.
#[cfg(feature = "real")]
const DATA_CHANNEL_CAPACITY: usize = 8;

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
/// Distribute a Mélodium treatment to a remote engine.
///
/// `DistributionEngine` connects to a distant engine using the provided access configuration,
/// negotiates protocol compatibility, then loads and launches the treatment identified by the
/// model parameters. It maintains an asynchronous connection and keeps track of distributed
/// instances and tracks, allowing the local engine to send inputs and receive outputs.
///
/// - `treatment`: fully-qualified identifier of the treatment to execute on the remote engine.
/// - `version`: version of the treatment (must be a valid SemVer string).
#[mel_model(
    param treatment string none
    param version string none
    continuous (continuous)
    shutdown shutdown
)]
pub struct DistributionEngine {
    model: Weak<DistributionEngineModel>,
    #[cfg(feature = "real")]
    protocol: AsyncRwLock<Option<AsyncArc<Protocol<NetworkStream>>>>,
    treatment: AsyncRwLock<Option<Arc<dyn Treatment>>>,
    tracks: AsyncRwLock<HashMap<u64, AsyncArc<AsyncRwLock<Track>>>>,
    start_attempted: AtomicBool,
    protocol_ready: Event,
    protocol_ready_fired: AtomicBool,
    stop_requested: AtomicBool,
    distant_run_id: AsyncRwLock<Option<Uuid>>,
}

impl DistributionEngine {
    fn new(model: Weak<DistributionEngineModel>) -> Self {
        Self {
            model,
            #[cfg(feature = "real")]
            protocol: AsyncRwLock::new(None),
            treatment: AsyncRwLock::new(None),
            tracks: AsyncRwLock::new(HashMap::new()),
            start_attempted: AtomicBool::new(false),
            protocol_ready: Event::new(),
            protocol_ready_fired: AtomicBool::new(false),
            stop_requested: AtomicBool::new(false),
            distant_run_id: AsyncRwLock::new(None),
        }
    }
}

#[cfg(feature = "real")]
impl DistributionEngine {
    fn fire_protocol_ready(&self) {
        self.protocol_ready_fired.store(true, Ordering::SeqCst);
        self.protocol_ready.notify(usize::MAX.additional());
    }

    async fn wait_protocol_ready(&self) {
        let listener = self.protocol_ready.listen();
        if self.protocol_ready_fired.load(Ordering::SeqCst) {
            return;
        }
        listener.await;
    }

    pub async fn fuse(&self) {
        if self.start_attempted.load(Ordering::SeqCst) {
            self.wait_protocol_ready().await;
        }
    }

    pub async fn start(
        &self,
        access: &work_mel::api::CommonAccess,
        params: HashMap<String, Value>,
    ) -> Result<(), String> {
        if self.start_attempted.swap(true, Ordering::SeqCst) {
            self.wait_protocol_ready().await;
            return Ok(());
        }

        let result = self.do_start(access, params).await;
        self.fire_protocol_ready();
        result
    }

    async fn do_start(
        &self,
        access: &work_mel::api::CommonAccess,
        params: HashMap<String, Value>,
    ) -> Result<(), String> {
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
                        asking_run_id: *melodium_engine::execution_run_id(),
                        group_id: *melodium_engine::execution_group_id(),
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
                                self.distant_run_id
                                    .write()
                                    .await
                                    .replace(confirm.confirming_run_id);
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
        // Only wait for a `start()` that has actually been attempted (or is in flight) to
        // settle before touching the protocol - mirrors the guard in `fuse()`. If `start()`
        // was never called (e.g. the `access` input closed without ever providing data,
        // because worker dispatch itself failed upstream), there is nothing that will ever
        // call `fire_protocol_ready()` for this instance, so waiting unconditionally here
        // would hang forever: `stop()` can be reached independently of `start()` (both the
        // `start` treatment's own failure fallback and the separate `stop` treatment can
        // call it), and previously did exactly that whenever a worker never got dispatched.
        if self.start_attempted.load(Ordering::SeqCst) {
            self.wait_protocol_ready().await;
        }

        if self.stop_requested.swap(true, Ordering::SeqCst) {
            return;
        }

        if let Some(protocol) = self.protocol.read().await.as_ref() {
            let _ = protocol.send_message(Message::Ended).await;
            protocol.close().await;
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
                    let (sender, receiver) = bounded(DATA_CHANNEL_CAPACITY);
                    inputs_senders.insert(name.clone(), sender);
                    inputs_receivers.insert(name.clone(), receiver);
                    io += 1;
                }

                for (name, _) in treatment.outputs() {
                    let (sender, receiver) = bounded(DATA_CHANNEL_CAPACITY);
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
                    // Split before constructing the message rather than after: each chunk
                    // becomes its own complete, self-contained InputData, so the receiving
                    // side (which already forwards each InputData's data independently,
                    // see `Message::InputData` handling in listen.rs) needs no reassembly
                    // logic at all — N smaller messages are transparently equivalent to
                    // this sender having flushed N smaller batches instead of one big one.
                    for chunk in chunk_raw_values(data.into(), max_batch_chunk_bytes()) {
                        if let Some(protocol) = self.protocol.read().await.as_ref() {
                            if let Err(_) = protocol
                                .send_message(Message::InputData(InputData {
                                    id: *distribution_id,
                                    name: name.clone(),
                                    data: chunk,
                                }))
                                .await
                            {
                                return Err(());
                            }
                        } else {
                            return Err(());
                        }
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
        let world = self.model.upgrade().map(|model| model.world().clone());

        // `start()` may never be called at all - e.g. the treatment
        // instance responsible for it never receives its `access` input, so
        // its track simply completes without ever calling `start`. Racing
        // against `wait_no_more_tracks` instead of only waiting on
        // `wait_protocol_ready` avoids blocking the whole engine forever in
        // that case: once no track is running or pending anymore, `start`
        // genuinely never will be called, so there is nothing left to wait
        // for and this continuous task can safely end.
        if let Some(world) = &world {
            select! {
                _ = self.wait_protocol_ready().fuse() => {},
                _ = world.wait_no_more_tracks().fuse() => {
                    if !self.protocol_ready_fired.load(Ordering::SeqCst) {
                        return;
                    }
                },
            }
        } else {
            self.wait_protocol_ready().await;
        }

        // Mirrors the server's concurrency-capped message dispatch (see `listen.rs`): without
        // it, a slow local consumer on *any one* distributed port blocks this single sequential
        // loop from processing messages for every other port on the same connection too, since
        // `output.send()` on the (now bounded, see `DATA_CHANNEL_CAPACITY`) per-port channel
        // can itself await. Dispatching each message as its own concurrently-polled future,
        // capped at `max_concurrent_messages()`, means one slow port only ever blocks up to
        // that many in-flight messages rather than the whole connection.
        let exec = async {
            if let Some(protocol) = self.protocol.read().await.as_ref().cloned() {
                // Plain `bool` locals can't be mutated from independent concurrently-polled
                // futures - shared, atomically-settable flags instead, mirroring how the
                // server keeps its own per-connection state behind shared interior mutability.
                let world = AsyncArc::new(world);
                let ended = AsyncArc::new(AtomicBool::new(false));
                let log_ended = AsyncArc::new(AtomicBool::new(false));
                let debug_ended = AsyncArc::new(AtomicBool::new(false));

                let manage_message = {
                    let protocol = AsyncArc::clone(&protocol);
                    let world = AsyncArc::clone(&world);
                    let ended = AsyncArc::clone(&ended);
                    let log_ended = AsyncArc::clone(&log_ended);
                    let debug_ended = AsyncArc::clone(&debug_ended);
                    move |msg: Message| {
                        let protocol = AsyncArc::clone(&protocol);
                        let world = AsyncArc::clone(&world);
                        let ended = AsyncArc::clone(&ended);
                        let log_ended = AsyncArc::clone(&log_ended);
                        let debug_ended = AsyncArc::clone(&debug_ended);
                        async move {
                            match msg {
                                Message::InstanciateStatus(instanciate_status) => {
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
                                Message::CloseInput(close_input) => {
                                    let track =
                                        self.tracks.read().await.get(&close_input.id).cloned();
                                    if let Some(track) = track {
                                        if let Some(input) = track
                                            .read()
                                            .await
                                            .inputs_receivers
                                            .get(&close_input.name)
                                        {
                                            input.close();
                                        }
                                    }
                                }
                                Message::OutputData(output_data) => {
                                    let track =
                                        self.tracks.read().await.get(&output_data.id).cloned();
                                    if let Some(track) = track {
                                        if let Some(output) = track
                                            .read()
                                            .await
                                            .outputs_senders
                                            .get(&output_data.name)
                                        {
                                            if output.send(output_data.data).await.is_err() {
                                                let _ = protocol
                                                    .send_message(Message::CloseOutput(
                                                        CloseOutput {
                                                            id: output_data.id,
                                                            name: output_data.name.clone(),
                                                        },
                                                    ))
                                                    .await;
                                            }
                                        }
                                    }
                                }
                                Message::CloseOutput(close_output) => {
                                    let track =
                                        self.tracks.read().await.get(&close_output.id).cloned();
                                    if let Some(track) = track {
                                        if let Some(output) = track
                                            .read()
                                            .await
                                            .outputs_senders
                                            .get(&close_output.name)
                                        {
                                            output.close();
                                        }
                                    }
                                }
                                Message::Log(log) => {
                                    if let Some(world) = world.as_ref() {
                                        let _ = world.inject_log(log).await;
                                    }
                                }
                                Message::Debug(debug) => {
                                    if let Some(world) = world.as_ref() {
                                        if let Some(run_id) =
                                            self.distant_run_id.read().await.as_ref()
                                        {
                                            let _ = world.inject_debug(*run_id, debug).await;
                                        }
                                    }
                                }
                                Message::Ended => {
                                    self.close_all().await;
                                    ended.store(true, Ordering::SeqCst);
                                }
                                Message::LogEnded => {
                                    log_ended.store(true, Ordering::SeqCst);
                                }
                                Message::DebugEnded => {
                                    debug_ended.store(true, Ordering::SeqCst);
                                }
                                Message::Probe => {}
                                _ => {}
                            }
                        }
                    }
                };

                let mut messages_futures = FuturesUnordered::new();

                // `OutputData`/`CloseOutput` messages for the same (id, name) port are read off
                // the wire in order, but dispatched as independent, concurrently-polled futures
                // below - `FuturesUnordered` makes no completion-order guarantee between them.
                // Since the per-port channel `OutputData` writes into is now bounded (see
                // `DATA_CHANNEL_CAPACITY`), its `send()` can genuinely suspend when the local
                // consumer is behind; if a same-port `CloseOutput` (whose `.close()` has no
                // await point of its own) gets polled first, it closes the channel out from
                // under that still-pending send, silently dropping the data. This map chains
                // same-key messages so each one explicitly awaits the previous one for that key
                // before doing its own work, restoring wire order without serializing unrelated
                // ports against each other.
                let mut port_chains = HashMap::new();

                // Unlike the server's equivalent unfold (which stops polling once it sees
                // `Message::Ended`, since that's genuinely the last thing a server connection
                // ever needs), this side keeps reading after `Ended` - it still needs to see
                // `LogEnded` before the exit condition below is satisfied, matching the
                // original sequential loop's `if ended && log_ended { break }`. Only a real
                // read error stops further polling.
                let unfold_protocol = unfold(true, |still_valid| {
                    let protocol = AsyncArc::clone(&protocol);
                    async move {
                        if still_valid {
                            match protocol.recv_message().await {
                                Err(err) => Some((Err(err), false)),
                                Ok(msg) => Some((Ok(msg), true)),
                            }
                        } else {
                            None
                        }
                    }
                })
                .fuse();

                pin_mut!(unfold_protocol);

                loop {
                    if messages_futures.len() < max_concurrent_messages() {
                        select! {
                            message = unfold_protocol.select_next_some() => {
                                match message {
                                    Err(_err) => {
                                        self.close_all().await;
                                        break;
                                    }
                                    Ok(msg) => {
                                        let key = match &msg {
                                            Message::OutputData(od) => {
                                                Some((od.id, od.name.clone()))
                                            }
                                            Message::CloseOutput(co) => {
                                                Some((co.id, co.name.clone()))
                                            }
                                            _ => None,
                                        };
                                        let prev =
                                            key.as_ref().and_then(|k| port_chains.get(k).cloned());
                                        let fut = manage_message(msg);
                                        let chained = async move {
                                            if let Some(prev) = prev {
                                                prev.await;
                                            }
                                            fut.await;
                                        }
                                        .boxed()
                                        .shared();
                                        if let Some(key) = key {
                                            port_chains.insert(key, chained.clone());
                                        }
                                        messages_futures.push(chained);
                                    }
                                }
                            }
                            () = messages_futures.select_next_some() => {}
                            complete => break,
                        }
                    } else {
                        // At the concurrency cap: stop pulling new messages off the socket
                        // until a handler finishes, same reasoning as the server's identical
                        // gate. `next()` only returns `None` on an empty stream, which cannot
                        // happen here since the `if` above guarantees at least one in-flight
                        // future.
                        if messages_futures.next().await.is_none() {
                            break;
                        }
                    }

                    // `messages_futures.is_empty()` matters here, not just the two flags:
                    // `LogEnded`'s handler is a single atomic store with no `.await` point, so
                    // it can complete before an earlier-arrived `Log`/`OutputData`/etc. handler
                    // that's still doing real async work. Breaking on the flags alone would
                    // drop whatever is still in `messages_futures` at that instant - silent
                    // data loss for messages that had already been read off the wire and
                    // dispatched, just hadn't finished processing yet. Requiring the queue to
                    // be drained first restores the sequential version's implicit guarantee
                    // that nothing still in flight gets abandoned.
                    if ended.load(Ordering::SeqCst)
                        && log_ended.load(Ordering::SeqCst)
                        && messages_futures.is_empty()
                    {
                        break;
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
                    // A single failed probe send doesn't prove the connection is dead: it
                    // can just as well be a transient hiccup over a real network (TLS
                    // renegotiation, a brief stall, ordinary internet jitter) on an otherwise
                    // healthy, still-working connection. Treating that one failure as "the
                    // worker is done" would silently close every channel exactly as if it
                    // had cleanly finished - with no way for anything downstream to tell the
                    // difference from a real completion. Give the connection a few more
                    // chances before actually giving up on it.
                    let mut attempts = 0;
                    let mut succeeded = false;
                    while attempts < PROBE_RETRY_ATTEMPTS {
                        if protocol.send_message(Message::Probe).await.is_ok() {
                            succeeded = true;
                            break;
                        }
                        attempts += 1;
                        if attempts < PROBE_RETRY_ATTEMPTS {
                            async_std::task::sleep(PROBE_RETRY_DELAY).await;
                        }
                    }
                    if !succeeded {
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
            self.fire_protocol_ready();
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

/// Treatment `start` for the `DistributionEngine` model.
///
/// This treatment is responsible for initiating the distribution
/// connection using the supplied `access` block, which must contain a single
/// `Access` value describing the remote addresses, port, keys and other
/// connection parameters. The treatment also takes arbitrary `params` that are
/// forwarded as launch parameters to the remote side.
///
/// On a successful start the treatment sends a unit token on `ready`.
/// If the engine cannot be started it emits a signal on `failed` followed by
/// an error message on `error` and triggers a fuse of the distributor so that
/// further attempts are ignored.
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
    if let Ok(access) = access.recv_one_as::<Arc<Access>>().await {
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

/// Treatment `stop` for the `DistributionEngine` model.
///
/// When the `trigger` block receives a unit token the treatment asks the
/// distributor to terminate its protocol connection and clean up any
/// resources. This allow the world to gracefully shut down the distributed
/// execution.
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

/// Treatment `distribute` for the `DistributionEngine` model.
///
/// When a unit token is received on `trigger`, this treatment requests a new
/// distributed track from the remote engine. If successful it waits for the
/// track to be fully instantiated and then sends its `distribution_id` on
/// the corresponding output. Failures during instantiation are reported via
/// the `failed` and `error` outputs.
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

/// Treatment `recv_stream` for receiving streaming output from a
/// distributed instance.
///
/// * `name` is the name of the output port defined by the distributed
///   treatment.
/// * `distribution_id` provides the identifier obtained from `distribute`.
///
/// The treatment forwards each value received from the remote output into the
/// `data` stream, converting raw values back into the requested generic type
/// `D`. If a value of the wrong datatype is encountered the stream is closed.
#[mel_treatment(
    model distributor DistributionEngine
    generic D (Deserialize)
    input distribution_id Block<u64>
    output data Stream<D>
)]
pub async fn recv_stream(name: string) {
    let datatype = D;

    #[cfg(feature = "real")]
    if let Ok(distribution_id) = distribution_id.recv_one_as::<u64>().await {
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

/// Treatment `recv_block` for receiving a single (blocking) output value
/// from a distributed instance.
///
/// It reads exactly one item from the remote output and emits it on the `data`
/// block. Afterwards the corresponding remote `name` output is closed.
#[mel_treatment(
    model distributor DistributionEngine
    generic D (Deserialize)
    input distribution_id Block<u64>
    output data Block<D>
)]
pub async fn recv_block(name: string) {
    let datatype = D;

    #[cfg(feature = "real")]
    if let Ok(distribution_id) = distribution_id.recv_one_as::<u64>().await {
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

/// Treatment `send_stream` for sending a stream of values to a distributed
/// instance input.
///
/// Values received on `data` are serialized and forwarded to the remote
/// treatment. The treatment handles automatic closing of the remote input when
/// the stream ends or an error occurs.
#[mel_treatment(
    model distributor DistributionEngine
    generic S (Serialize)
    input distribution_id Block<u64>
    input data Stream<S>
)]
pub async fn send_stream(name: string) {
    #[cfg(feature = "real")]
    if let Ok(distribution_id) = distribution_id.recv_one_as::<u64>().await {
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

/// Treatment `send_block` for sending a single value to a distributed input.
///
/// The provided `data` block is consumed once, serialized and sent to the
/// remote side. The remote input is closed after transmission.
#[mel_treatment(
    model distributor DistributionEngine
    generic S (Serialize)
    input distribution_id Block<u64>
    input data Block<S>
)]
pub async fn send_block(name: string) {
    #[cfg(feature = "real")]
    if let Ok(distribution_id) = distribution_id.recv_one_as::<u64>().await {
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

#[cfg(all(test, feature = "real"))]
mod graceful_shutdown_drain_tests {
    use async_std::channel::bounded as async_bounded;
    use async_std::sync::Arc as AsyncArc;
    use core::future::Future;
    use core::pin::Pin;
    use core::sync::atomic::{AtomicBool, Ordering};
    use core::time::Duration;
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;

    type BoxFut = Pin<Box<dyn Future<Output = ()>>>;

    // Mirrors the exact drain-before-exit guard used in `continuous()`'s message loop:
    // the loop must not consider itself done just because a terminal flag flipped, it
    // also has to wait for every already-dispatched handler in `messages_futures` to
    // actually finish.
    async fn drain_until_done(messages_futures: &mut FuturesUnordered<BoxFut>, done: &AtomicBool) {
        while !(done.load(Ordering::SeqCst) && messages_futures.is_empty()) {
            messages_futures.next().await;
        }
    }

    // Reproduces the race that caused the bug: `Message::LogEnded`'s handler is a
    // trivial, immediately-ready atomic store, while a `Message::Log` handler dispatched
    // just before it does genuine async work (here, waiting on a channel) and can still
    // be pending when `log_ended` flips true. A drain guard that only checks the flag
    // would exit and silently drop the still-pending handler; checking
    // `messages_futures.is_empty()` as well must keep the loop alive until it finishes.
    #[test]
    fn drain_waits_for_slow_handler_still_pending_when_flag_flips() {
        async_std::task::block_on(async {
            let (unblock_tx, unblock_rx) = async_bounded::<()>(1);
            let processed = AsyncArc::new(AtomicBool::new(false));
            let log_ended = AsyncArc::new(AtomicBool::new(false));

            let mut messages_futures: FuturesUnordered<BoxFut> = FuturesUnordered::new();

            // Stands in for Message::Log's handler: doesn't complete until told to.
            {
                let processed = AsyncArc::clone(&processed);
                messages_futures.push(Box::pin(async move {
                    let _ = unblock_rx.recv().await;
                    processed.store(true, Ordering::SeqCst);
                }));
            }

            // Stands in for Message::LogEnded's handler: completes immediately.
            {
                let log_ended = AsyncArc::clone(&log_ended);
                messages_futures.push(Box::pin(async move {
                    log_ended.store(true, Ordering::SeqCst);
                }));
            }

            // The drain must not be able to finish while the slow handler is still
            // blocked, even though `log_ended` will already be true almost immediately.
            let still_pending = async_std::future::timeout(
                Duration::from_millis(200),
                drain_until_done(&mut messages_futures, &log_ended),
            )
            .await
            .is_err();
            assert!(
                still_pending,
                "drain must keep waiting while a dispatched handler has not completed yet, \
                 not exit as soon as the flag is set"
            );
            assert!(
                !processed.load(Ordering::SeqCst),
                "the slow handler must not have been abandoned"
            );

            // Once the slow handler is allowed to finish, the drain must complete and the
            // handler must have actually run - nothing was silently dropped.
            unblock_tx.send(()).await.unwrap();
            async_std::future::timeout(
                Duration::from_secs(1),
                drain_until_done(&mut messages_futures, &log_ended),
            )
            .await
            .expect("drain should complete promptly once the pending handler is unblocked");
            assert!(
                processed.load(Ordering::SeqCst),
                "the slow handler must have run to completion before the drain considered \
                 itself done"
            );
        });
    }
}

mel_package!();
