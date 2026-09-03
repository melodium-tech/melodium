use crate::error::DistributionResult;
use crate::framing::{chunk_raw_values, max_batch_chunk_bytes};
use crate::protocol::Protocol;
use crate::{messages, messages::*, VERSION};
use async_std::channel::{bounded, Sender};
use async_std::sync::Barrier;
use async_std::{
    future::timeout,
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::RwLock as AsyncRwLock,
};
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use futures::stream::{unfold, FuturesUnordered};
use futures::{pin_mut, select, FutureExt, StreamExt};
use futures_rustls::TlsAcceptor;
use melodium_common::executive::{Level, Log};
use melodium_common::{
    descriptor::{Entry, Identifier, Model as CommonModel, Treatment as CommonTreatment, Version},
    executive::{ResultStatus, TransmissionValue, Value},
};
use melodium_engine::debug::{DebugLevel, Event};
use melodium_engine::descriptor::{Model, Treatment};
use melodium_engine::execution_group_id;
use melodium_loader::Loader;
use melodium_share::{ProgramDump, RawValue, SharingError, SharingResult};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, OnceLock},
};
use uuid::Uuid;

const CERTIFICATE_CHAIN: &[u8; 4715] = include_bytes!("../melodium-chain.pem");
const LOCALHOST_KEY: &[u8; 3272] = include_bytes!("../melodium-localhost.key.pem");
const DEFAULT_TEARDOWN_TIMEOUT_SECS: u64 = 60;
/// Number of attempts made to send a keepalive `Probe` before treating the connection as
/// genuinely dead. See the comment at the probe retry loop for why a single failure isn't
/// trusted on its own.
const PROBE_RETRY_ATTEMPTS: u32 = 3;
/// Delay between probe retry attempts.
const PROBE_RETRY_DELAY: Duration = Duration::from_secs(2);

/// Grace period, after the distributed engine and connection are expected to
/// be done, before forcing teardown of the connection and log/debug channels.
/// Overridable through `MELODIUM_DIST_TEARDOWN_TIMEOUT_SECS`, mainly to allow
/// tests to exercise this safety net without waiting a full minute.
fn teardown_timeout() -> Duration {
    static TEARDOWN_TIMEOUT: OnceLock<Duration> = OnceLock::new();
    *TEARDOWN_TIMEOUT.get_or_init(|| {
        std::env::var("MELODIUM_DIST_TEARDOWN_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(DEFAULT_TEARDOWN_TIMEOUT_SECS))
    })
}

const DEFAULT_LOG_CHANNEL_CAPACITY: usize = 256;

/// Bound on how many `Log` events may be queued for this connection's log listener before the
/// engine's own log fan-out task (which awaits each listener's `send`) starts waiting on it.
/// That wait only stalls log/debug *delivery* for this connection, never treatment execution -
/// see the log/debug listener setup below for why. Overridable through
/// `MELODIUM_DIST_LOG_CHANNEL_CAPACITY`.
fn log_channel_capacity() -> usize {
    static CAPACITY: OnceLock<usize> = OnceLock::new();
    *CAPACITY.get_or_init(|| {
        std::env::var("MELODIUM_DIST_LOG_CHANNEL_CAPACITY")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_LOG_CHANNEL_CAPACITY)
    })
}

const DEFAULT_DEBUG_CHANNEL_CAPACITY: usize = 256;

/// Same rationale as `log_channel_capacity`, for debug events. Overridable through
/// `MELODIUM_DIST_DEBUG_CHANNEL_CAPACITY`.
fn debug_channel_capacity() -> usize {
    static CAPACITY: OnceLock<usize> = OnceLock::new();
    *CAPACITY.get_or_init(|| {
        std::env::var("MELODIUM_DIST_DEBUG_CHANNEL_CAPACITY")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_DEBUG_CHANNEL_CAPACITY)
    })
}

const DEFAULT_MAX_CONCURRENT_MESSAGES: usize = 64;

/// Upper bound on how many incoming protocol messages may be decoded and awaiting handling at
/// once for a single connection. Without this, a fast peer (or a locally slow consumer) lets
/// the message-read loop keep decoding indefinitely, piling up in-memory message payloads with
/// no back-pressure back to the socket. Once this many handlers are in flight, the read loop
/// stops pulling new messages until one finishes - so a slow consumer now visibly throttles the
/// connection's read side instead of growing memory. Overridable through
/// `MELODIUM_DIST_MAX_CONCURRENT_MESSAGES`.
///
/// Shared between the server's message loop here and the client's in `distrib-mel` (which
/// mirrors this same dispatch pattern), so `pub` and re-exported from the crate root — one
/// setting governs the same policy on both sides of the connection.
pub fn max_concurrent_messages() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("MELODIUM_DIST_MAX_CONCURRENT_MESSAGES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_MAX_CONCURRENT_MESSAGES)
            .max(1)
    })
}

/// Returns `true` if the run actually launched and went through its lifecycle; `false` if
/// launch never happened. `ended`, if given, is called once that lifecycle is genuinely over
/// (protocol/engine teardown complete), before this function returns - mirroring how the
/// caller's own logs/debug monitoring is drained afterward, so a caller reporting run status
/// downstream (e.g. to an API) sees "ended" exactly when this connection's work is done, not
/// deferred behind unrelated bookkeeping.
pub async fn launch_listen(
    bind: SocketAddr,
    certificate_chain: &[u8],
    key: &[u8],
    version: &Version,
    expect_key: Uuid,
    emit_key: Uuid,
    loader: Loader,
    wait_for: Option<Duration>,
    max_duration: Option<Duration>,
    logs_senders: Vec<Sender<Log>>,
    debug_senders: Vec<Sender<Event>>,
    program_dump_sender: Option<Sender<ProgramDump>>,
    launched: Option<
        Box<
            dyn FnOnce(
                Result<(), String>,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
        >,
    >,
    ended: Option<
        Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>>,
    >,
) -> bool {
    let acceptor = acceptor(certificate_chain, key).unwrap();
    let listener = TcpListener::bind(bind).await.unwrap();

    let accept_stream = async {
        loop {
            if let Ok((stream, _addr)) = listener.accept().await {
                if let Ok(stream) = acceptor.accept(stream).await {
                    return stream;
                }
            }
        }
    };

    let stream = if let Some(wait_for) = wait_for {
        match timeout(wait_for, accept_stream).await {
            Ok(stream) => stream,
            Err(_) => {
                if let Some(launched) = launched {
                    launched(Err("Distribution timeout".to_string())).await;
                }
                return false;
            }
        }
    } else {
        accept_stream.await
    };

    launch_listen_stream(
        stream,
        version,
        expect_key,
        emit_key,
        loader,
        max_duration,
        logs_senders,
        debug_senders,
        program_dump_sender,
        launched,
        ended,
    )
    .await
}

/// Returns `true` if the run actually launched and went through its lifecycle; `false` if
/// launch never happened. `ended`, if given, is called once that lifecycle is genuinely over,
/// before this function returns.
pub async fn launch_listen_localcert(
    bind: SocketAddr,
    version: &Version,
    expect_key: Uuid,
    emit_key: Uuid,
    loader: Loader,
    wait_for: Option<Duration>,
    max_duration: Option<Duration>,
    logs_senders: Vec<Sender<Log>>,
    debug_senders: Vec<Sender<Event>>,
    program_dump_sender: Option<Sender<ProgramDump>>,
    launched: Option<
        Box<
            dyn FnOnce(
                Result<(), String>,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
        >,
    >,
    ended: Option<
        Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>>,
    >,
) -> bool {
    launch_listen(
        bind,
        CERTIFICATE_CHAIN.as_slice(),
        LOCALHOST_KEY.as_slice(),
        version,
        expect_key,
        emit_key,
        loader,
        wait_for,
        max_duration,
        logs_senders,
        debug_senders,
        program_dump_sender,
        launched,
        ended,
    )
    .await
}

/// Returns `true` if the run actually launched and went through its lifecycle; `false` if
/// launch never happened. `ended`, if given, is called once that lifecycle is genuinely over,
/// before this function returns.
pub async fn launch_listen_unsecure(
    bind: SocketAddr,
    version: &Version,
    expect_key: Uuid,
    emit_key: Uuid,
    loader: Loader,
    wait_for: Option<Duration>,
    max_duration: Option<Duration>,
    logs_senders: Vec<Sender<Log>>,
    debug_senders: Vec<Sender<Event>>,
    program_dump_sender: Option<Sender<ProgramDump>>,
    launched: Option<
        Box<
            dyn FnOnce(
                Result<(), String>,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
        >,
    >,
    ended: Option<
        Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>>,
    >,
) -> bool {
    let listener = TcpListener::bind(bind).await.unwrap();

    let accept_stream = async {
        let (stream, _addr) = listener.accept().await.unwrap();

        stream
    };

    let stream = if let Some(wait_for) = wait_for {
        match timeout(wait_for, accept_stream).await {
            Ok(stream) => stream,
            Err(_) => {
                if let Some(launched) = launched {
                    launched(Err("Distribution timeout".to_string())).await;
                }
                return false;
            }
        }
    } else {
        accept_stream.await
    };

    launch_listen_stream(
        stream,
        version,
        expect_key,
        emit_key,
        loader,
        max_duration,
        logs_senders,
        debug_senders,
        program_dump_sender,
        launched,
        ended,
    )
    .await
}

/// Returns `true` if the run actually launched and went through its lifecycle; `false` if
/// launch never happened. `ended`, if given, is called once that lifecycle is genuinely over
/// (protocol/engine teardown complete), before this function returns.
async fn launch_listen_stream<S: Read + Write + Unpin + Send + 'static>(
    stream: S,
    version: &Version,
    expect_key: Uuid,
    emit_key: Uuid,
    loader: Loader,
    max_duration: Option<Duration>,
    logs_senders: Vec<Sender<Log>>,
    debug_senders: Vec<Sender<Event>>,
    program_dump_sender: Option<Sender<ProgramDump>>,
    launched: Option<
        Box<
            dyn FnOnce(
                Result<(), String>,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
        >,
    >,
    ended: Option<
        Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>>,
    >,
) -> bool {
    let protocol = Arc::new(Protocol::new(stream));

    match protocol.recv_message().await {
        Ok(Message::AskDistribution(ask)) => {
            let accept = &ask.melodium_version == version
                && ask.distribution_version == VERSION
                && ask.key == expect_key
                && &ask.group_id == execution_group_id();
            protocol
                .send_message(Message::ConfirmDistribution(ConfirmDistribution {
                    melodium_version: version.clone(),
                    distribution_version: VERSION.clone(),
                    key: emit_key,
                    accept,
                    confirming_run_id: *melodium_engine::execution_run_id(),
                    group_id: *melodium_engine::execution_group_id(),
                }))
                .await
                .unwrap();

            if !accept {
                if let Some(launched) = launched {
                    launched(Err("Distribution refused".to_string())).await;
                }
                return false;
            }
        }
        _ => {
            if let Some(launched) = launched {
                launched(Err("No distribution asked".to_string())).await;
            }
            return false;
        }
    }

    let (distributed_collection, entrypoint, parameters) = match protocol.recv_message().await {
        Ok(Message::LoadAndLaunch(lal)) => {
            if let Some(program_dump_sender) = program_dump_sender {
                let _ = program_dump_sender
                    .send(ProgramDump {
                        collection: lal.collection.clone(),
                        entrypoint: lal.entrypoint.clone(),
                        parameters: lal
                            .parameters
                            .iter()
                            .map(|(name, value)| (name.clone(), value.clone()))
                            .collect(),
                    })
                    .await;
                program_dump_sender.close();
            }
            (lal.collection, lal.entrypoint, lal.parameters)
        }
        _ => {
            if let Some(launched) = launched {
                launched(Err("No load provided".to_string())).await;
            }
            return false;
        }
    };

    // Proceed to load of compiled elements
    let mut result = DistributionResult::new_success(());
    for element in distributed_collection.elements() {
        if element.is_compiled() {
            if let Ok(identifier) = TryInto::<Identifier>::try_into(element.identifier()) {
                let _: Option<Identifier> = result.merge_degrade_failure(DistributionResult::from(
                    loader.load(&identifier.into()),
                ));
            } else {
                result = result.and_degrade_failure(DistributionResult::from::<(), _, _>(
                    SharingResult::new_failure(SharingError::invalid_identifier(
                        18,
                        element.identifier().clone(),
                    )),
                ));
            }
        }
    }

    if let Err(fail) = result.as_result() {
        protocol
            .send_message(Message::LaunchStatus(messages::LaunchStatus::Failure(
                fail.to_string(),
            )))
            .await
            .unwrap();
    }

    let mut collection = loader.collection().clone();

    // Proceed descriptor build
    for element in distributed_collection.elements() {
        if !element.is_compiled() {
            match element {
                melodium_share::Element::Model(m) => {
                    let model: Option<Arc<Model>> = result.merge_degrade_failure(
                        DistributionResult::from(m.make_descriptor(&collection)),
                    );
                    if let Some(model) = model {
                        collection.insert(Entry::Model(Arc::clone(&model) as Arc<dyn CommonModel>));
                    }
                }
                melodium_share::Element::Treatment(t) => {
                    let treatment: Option<Arc<Treatment>> = result.merge_degrade_failure(
                        DistributionResult::from(t.make_descriptor(&collection)),
                    );
                    if let Some(treatment) = treatment {
                        collection.insert(Entry::Treatment(
                            Arc::clone(&treatment) as Arc<dyn CommonTreatment>
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    let collection = Arc::new(collection);

    // Proceed to design
    for element in distributed_collection.elements() {
        if !element.is_compiled() {
            match element {
                melodium_share::Element::Model(m) => {
                    result = result
                        .and_degrade_failure(DistributionResult::from(m.make_design(&collection)));
                }
                melodium_share::Element::Treatment(t) => {
                    result = result
                        .and_degrade_failure(DistributionResult::from(t.make_design(&collection)));
                }
                _ => {}
            }
        }
    }

    // Give it to engine
    let parameters = parameters
        .into_iter()
        .map(|(name, val)| (name, val.to_value(&collection).unwrap()))
        .collect();
    let engine =
        melodium_engine::new_engine(Arc::clone(&collection), Level::Trace, DebugLevel::Detailed);
    engine.set_auto_end(false);

    let (logs_sender, logs_receiver) = bounded(log_channel_capacity());
    engine.add_logs_listener(logs_sender);
    for log_sender in logs_senders {
        engine.add_logs_listener(log_sender);
    }
    let watchdog_logs_receiver = logs_receiver.clone();

    let (debug_sender, debug_receiver) = bounded(debug_channel_capacity());
    engine.add_debug_listener(debug_sender);
    for debug_sender in debug_senders {
        engine.add_debug_listener(debug_sender);
    }
    let watchdog_debug_receiver = debug_receiver.clone();

    if let Err(fail) = engine
        .genesis(&entrypoint.try_into().unwrap(), parameters)
        .as_result()
    {
        protocol
            .send_message(Message::LaunchStatus(messages::LaunchStatus::Failure(
                fail.to_string(),
            )))
            .await
            .unwrap();
        if let Some(launched) = launched {
            launched(Err(fail.to_string())).await;
        }
        return false;
    }

    protocol
        .send_message(Message::LaunchStatus(messages::LaunchStatus::Ok))
        .await
        .unwrap();

    if let Some(launched) = launched {
        launched(Ok(())).await;
    }

    let barrier = Arc::new(Barrier::new(2));
    let expired = Arc::new(AtomicBool::new(false));
    let limit = {
        let engine = Arc::clone(&engine);
        let barrier = Arc::clone(&barrier);
        let expired = Arc::clone(&expired);
        async move {
            if let Some(max_duration) = max_duration {
                futures::future::select_all([
                    async {
                        barrier.wait().await;
                    }
                    .boxed(),
                    async {
                        async_std::task::sleep(max_duration).await;
                        expired.store(true, core::sync::atomic::Ordering::Relaxed);
                    }
                    .boxed(),
                ])
                .await;
                engine.end().await;
            } else {
                barrier.wait().await;
            }
        }
    };
    let live = {
        let engine = Arc::clone(&engine);
        let protocol = Arc::clone(&protocol);
        async move {
            engine.live().await;
            let _ = protocol.send_message(Message::Ended).await;
            if !expired.load(core::sync::atomic::Ordering::Relaxed) {
                barrier.wait().await;
            }
        }
    };
    let run = async {
        let engine = Arc::clone(&engine);
        let protocol = Arc::clone(&protocol);
        let collection = Arc::clone(&collection);

        let tracks_entry_outputs = Arc::new(AsyncRwLock::new(HashMap::new()));
        let tracks_entry_inputs = Arc::new(AsyncRwLock::new(HashMap::new()));

        let manage_message = {
            let protocol = Arc::clone(&protocol);
            let engine = Arc::clone(&engine);
            let collection = Arc::clone(&collection);
            let tracks_entry_outputs = Arc::clone(&tracks_entry_outputs);
            move |message| {
                let protocol = Arc::clone(&protocol);
                let engine = Arc::clone(&engine);
                let collection = Arc::clone(&collection);
                let tracks_entry_outputs = Arc::clone(&tracks_entry_outputs);
                let tracks_entry_inputs = Arc::clone(&tracks_entry_inputs);
                async move {
                    match message {
                        Message::Instanciate(instanciate) => {
                            let protocol = Arc::clone(&protocol);
                            let tracks_entry_outputs = Arc::clone(&tracks_entry_outputs);
                            let tracks_entry_inputs = Arc::clone(&tracks_entry_inputs);
                            let track_id = instanciate.id;

                            if let Err(failure) = engine
                                .instanciate(Some(Box::new({
                                    let protocol = Arc::clone(&protocol);
                                    move |entry_outputs, entry_inputs| {
                                        let mut inputs_management = Vec::new();
                                        let mut inputs_storage = HashMap::new();
                                        for (name, input) in entry_inputs {
                                            let protocol = Arc::clone(&protocol);
                                            let input = Arc::new(input);
                                            inputs_storage.insert(name.clone(), Arc::clone(&input));
                                            let listener = async move {
                                                'recv: while let Ok(data) = input.recv_many().await
                                                {
                                                    let data: Vec<RawValue> =
                                                        Into::<VecDeque<Value>>::into(data)
                                                            .into_iter()
                                                            .map(|val| val.into())
                                                            .collect();

                                                    // Split before constructing the
                                                    // message, same reasoning as
                                                    // `distrib-mel`'s `send_data`: each
                                                    // chunk is a complete, independent
                                                    // OutputData, and the receiving side
                                                    // already forwards each one on its
                                                    // own (see `Message::OutputData`
                                                    // handling), so no reassembly is
                                                    // needed on receipt.
                                                    for chunk in chunk_raw_values(
                                                        data,
                                                        max_batch_chunk_bytes(),
                                                    ) {
                                                        if protocol
                                                            .send_message(Message::OutputData(
                                                                OutputData {
                                                                    id: track_id,
                                                                    name: name.clone(),
                                                                    data: chunk,
                                                                },
                                                            ))
                                                            .await
                                                            .is_err()
                                                        {
                                                            input.close();
                                                            break 'recv;
                                                        }
                                                    }
                                                }
                                                let _ = protocol
                                                    .send_message(Message::CloseOutput(
                                                        CloseOutput {
                                                            id: track_id,
                                                            name: name.clone(),
                                                        },
                                                    ))
                                                    .await;
                                            };
                                            inputs_management.push(Box::new(Box::pin(listener)));
                                        }

                                        let protocol = Arc::clone(&protocol);
                                        vec![Box::new(Box::pin(async move {
                                            {
                                                tracks_entry_inputs
                                                    .write()
                                                    .await
                                                    .insert(track_id, inputs_storage);

                                                tracks_entry_outputs
                                                    .write()
                                                    .await
                                                    .insert(track_id, entry_outputs);
                                            }

                                            let _ = protocol
                                                .send_message(Message::InstanciateStatus(
                                                    InstanciateStatus::Ok { id: track_id },
                                                ))
                                                .await;

                                            futures::future::join_all(inputs_management).await;

                                            ResultStatus::Ok
                                        }))]
                                    }
                                })))
                                .await
                                .as_result()
                            {
                                let _ = protocol
                                    .send_message(Message::InstanciateStatus(
                                        InstanciateStatus::Failure {
                                            id: track_id,
                                            message: failure.to_string(),
                                        },
                                    ))
                                    .await;
                            }
                        }
                        Message::InputData(input_data) => {
                            if let Some(outputs) =
                                tracks_entry_outputs.read().await.get(&input_data.id)
                            {
                                if let Some(output) = outputs.get(&input_data.name) {
                                    match output
                                        .send_many(TransmissionValue::Other(
                                            input_data
                                                .data
                                                .into_iter()
                                                .map(|val| val.to_value(&collection).unwrap())
                                                .collect::<VecDeque<Value>>(),
                                        ))
                                        .await
                                    {
                                        Ok(_) => {}
                                        Err(_) => {
                                            let _ = protocol
                                                .send_message(Message::CloseInput(CloseInput {
                                                    id: input_data.id,
                                                    name: input_data.name.clone(),
                                                }))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                        Message::CloseInput(close_input) => {
                            if let Some(outputs) =
                                tracks_entry_outputs.read().await.get(&close_input.id)
                            {
                                if let Some(output) = outputs.get(&close_input.name) {
                                    output.close().await;
                                }
                            }
                        }
                        Message::CloseOutput(close_output) => {
                            if let Some(inputs) =
                                tracks_entry_inputs.read().await.get(&close_output.id)
                            {
                                if let Some(input) = inputs.get(&close_output.name) {
                                    input.close();
                                }
                            }
                        }
                        Message::Ended => {
                            for (_, outputs) in tracks_entry_outputs.read().await.iter() {
                                for (_, output) in outputs {
                                    output.close().await;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        };

        let mut messages_futures = FuturesUnordered::new();

        let unfold_protocol = unfold(true, |still_valid| {
            let protocol = Arc::clone(&protocol);
            async move {
                if still_valid {
                    match protocol.recv_message().await {
                        Ok(Message::Ended) => Some((Ok(Message::Ended), false)),
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
                            Ok(Message::Ended) => {
                                break;
                            }
                            Err(_err) => {
                                break;
                            }
                            Ok(msg) => {
                                messages_futures.push(manage_message(msg));
                            }
                        }
                    }
                    () = messages_futures.select_next_some() => {}
                    complete => break,
                }
            } else {
                // At the concurrency cap: stop pulling new messages off the socket until a
                // handler finishes, so a locally slow consumer throttles this connection's read
                // side instead of letting decoded message payloads pile up in memory. `next()`
                // only returns `None` on an empty stream, which cannot happen here since the
                // `if` above guarantees at least one in-flight future.
                if messages_futures.next().await.is_none() {
                    break;
                }
            }
        }

        for (_, outputs) in tracks_entry_outputs.read().await.iter() {
            for (_, output) in outputs {
                output.close().await;
            }
        }
        engine.end().await;
    };
    let logs = {
        let protocol = Arc::clone(&protocol);

        async move {
            while let Ok(log) = logs_receiver.recv().await {
                if protocol.send_message(Message::Log(log)).await.is_err() {
                    break;
                }
            }
            let _ = protocol.send_message(Message::LogEnded).await;
        }
    };
    let debug = {
        let protocol = Arc::clone(&protocol);

        async move {
            while let Ok(event) = debug_receiver.recv().await {
                if protocol
                    .send_message(Message::Debug(
                        serde_json::to_string(&melodium_share::Event::from(&event))
                            .unwrap_or_else(|_| "\"<failed to serialize debug event>\"".to_string())
                            .into(),
                    ))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            let _ = protocol.send_message(Message::DebugEnded).await;
        }
    };
    let probe = {
        let engine = Arc::clone(&engine);
        let protocol = Arc::clone(&protocol);
        async move {
            loop {
                async_std::task::sleep(Duration::from_secs(10)).await;
                // A single failed probe send doesn't prove the connection is dead: it can
                // just as well be a transient hiccup over a real network (TLS
                // renegotiation, a brief stall, ordinary internet jitter) on an otherwise
                // healthy connection. Forcibly ending the engine's own still-running work
                // over that alone would cut a genuinely in-progress job short. Give the
                // connection a few more chances before actually giving up on it.
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
                    engine.end().await;
                    break;
                }
            }
            protocol.close().await;
        }
    };

    let probe = async_std::task::spawn(probe);

    // `logs`/`debug` are spawned as independent tasks, not joined alongside
    // `limit`/`live`/`run` below: they must be actively polled and forwarding to the
    // peer for the whole duration of the run, not just after the real work is done -
    // otherwise log/debug events only get sent once the job already finished, instead
    // of arriving live while it's in progress.
    let logs = async_std::task::spawn(logs);
    let debug = async_std::task::spawn(debug);

    // `limit`/`live`/`run` complete once real work is genuinely done: `max_duration`
    // elapsing, the engine reporting itself ended, or the peer confirming `Ended` /
    // the protocol erroring out (including via the probe retry loop above giving up).
    // None of them are time-bounded by anything shorter than the run's own
    // `max_duration`, so joining them here can legitimately take as long as the
    // distributed work does - a job that runs for hours must not have its connection
    // torn out from under it by an unrelated fixed timer.
    futures::join!(limit, live, run);

    // Only past this point do we bound how long we wait for `logs`/`debug` to finish
    // flushing whatever's left: `engine.end()` has already run (via `limit`/`live`/`run`
    // above), which closes the listeners those two tasks are reading from, so they
    // should wrap up quickly on their own. If the peer never confirms and the
    // connection stays silently open, they could otherwise hang forever - so only this
    // remaining, genuinely-bounded tail is raced against the watchdog, instead of the
    // watchdog racing the actual distributed work from the start.
    if timeout(teardown_timeout(), async {
        futures::join!(logs, debug);
    })
    .await
    .is_err()
    {
        engine.end().await;
        protocol.close().await;
        watchdog_logs_receiver.close();
        watchdog_debug_receiver.close();
    }

    protocol.close().await;
    probe.cancel().await;

    if let Some(ended) = ended {
        ended().await;
    }

    true
}

fn acceptor(
    mut certificate_chain: &[u8],
    mut key: &[u8],
) -> Result<TlsAcceptor, Box<dyn std::error::Error>> {
    let certs = rustls_pemfile::certs(&mut certificate_chain)
        .filter_map(|res| res.ok())
        .collect();
    let key = rustls_pemfile::pkcs8_private_keys(&mut key)
        .next()
        .unwrap()?;

    Ok(TlsAcceptor::from(Arc::new(
        futures_rustls::rustls::ServerConfig::builder_with_protocol_versions(&[
            &futures_rustls::rustls::version::TLS13,
        ])
        .with_no_client_auth()
        .with_single_cert(certs, futures_rustls::pki_types::PrivateKeyDer::Pkcs8(key))?,
    )))
}
