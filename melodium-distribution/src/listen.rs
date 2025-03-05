use crate::error::DistributionResult;
use crate::protocol::Protocol;
use crate::{messages, messages::*, VERSION};
#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
use async_native_tls::TlsAcceptor;
use async_std::sync::Barrier;
use async_std::{
    future::timeout,
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::RwLock as AsyncRwLock,
};
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use futures::FutureExt;
#[cfg(any(
    all(not(target_os = "windows"), not(target_vendor = "apple")),
    all(target_os = "windows", target_env = "gnu")
))]
use futures_rustls::TlsAcceptor;
use melodium_common::{
    descriptor::{Entry, Identifier, Model as CommonModel, Treatment as CommonTreatment, Version},
    executive::{ResultStatus, TransmissionValue, Value},
};
use melodium_engine::descriptor::{Model, Treatment};
use melodium_loader::Loader;
use melodium_share::{SharingError, SharingResult};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use uuid::Uuid;

const CERTIFICATE_CHAIN: &[u8; 4715] = include_bytes!("../melodium-chain.pem");
const LOCALHOST_KEY: &[u8; 3272] = include_bytes!("../melodium-localhost.key.pem");

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
) {
    let acceptor = acceptor(certificate_chain, key).unwrap();
    let listener = TcpListener::bind(bind).await.unwrap();

    let accept_stream = async {
        let (stream, _addr) = listener.accept().await.unwrap();

        acceptor.accept(stream).await.unwrap()
    };

    let stream = if let Some(wait_for) = wait_for {
        match timeout(wait_for, accept_stream).await {
            Ok(stream) => stream,
            Err(_) => return,
        }
    } else {
        accept_stream.await
    };

    launch_listen_stream(stream, version, expect_key, emit_key, loader, max_duration).await
}

pub async fn launch_listen_localcert(
    bind: SocketAddr,
    version: &Version,
    expect_key: Uuid,
    emit_key: Uuid,
    loader: Loader,
    wait_for: Option<Duration>,
    max_duration: Option<Duration>,
) {
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
    )
    .await
}

pub async fn launch_listen_unsecure(
    bind: SocketAddr,
    version: &Version,
    expect_key: Uuid,
    emit_key: Uuid,
    loader: Loader,
    wait_for: Option<Duration>,
    max_duration: Option<Duration>,
) {
    let listener = TcpListener::bind(bind).await.unwrap();

    let accept_stream = async {
        let (stream, _addr) = listener.accept().await.unwrap();

        stream
    };

    let stream = if let Some(wait_for) = wait_for {
        match timeout(wait_for, accept_stream).await {
            Ok(stream) => stream,
            Err(_) => return,
        }
    } else {
        accept_stream.await
    };

    launch_listen_stream(stream, version, expect_key, emit_key, loader, max_duration).await
}

async fn launch_listen_stream<S: Read + Write + Unpin + Send + 'static>(
    stream: S,
    version: &Version,
    expect_key: Uuid,
    emit_key: Uuid,
    loader: Loader,
    max_duration: Option<Duration>,
) {
    let protocol = Arc::new(Protocol::new(stream));

    match protocol.recv_message().await {
        Ok(Message::AskDistribution(ask)) => {
            let accept = &ask.melodium_version == version
                && ask.distribution_version == VERSION
                && ask.key == expect_key;
            protocol
                .send_message(Message::ConfirmDistribution(ConfirmDistribution {
                    melodium_version: version.clone(),
                    distribution_version: VERSION.clone(),
                    key: emit_key,
                    accept,
                }))
                .await
                .unwrap();

            if !accept {
                return;
            }
        }
        _ => return,
    }

    let (distributed_collection, entrypoint, parameters) = match protocol.recv_message().await {
        Ok(Message::LoadAndLaunch(lal)) => (lal.collection, lal.entrypoint, lal.parameters),
        _ => return,
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
                result = result.and_degrade_failure(DistributionResult::from(
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
    let engine = melodium_engine::new_engine(Arc::clone(&collection));
    engine.set_auto_end(false);
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
    }

    protocol
        .send_message(Message::LaunchStatus(messages::LaunchStatus::Ok))
        .await
        .unwrap();

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
                        eprintln!("Awaiting barrier (0)");
                        barrier.wait().await;
                        eprintln!("Awaited barrier (0) {:?}", std::time::SystemTime::now());
                    }
                    .boxed(),
                    async {
                        async_std::task::sleep(max_duration).await;
                        expired.store(true, core::sync::atomic::Ordering::Relaxed);
                    }
                    .boxed(),
                ])
                .await;
                eprintln!("Ending engine (1)");
                engine.end().await;
                eprintln!("Ended engine (1) {:?}", std::time::SystemTime::now());
            } else {
                eprintln!("Awaiting barrier (1)");
                barrier.wait().await;
                eprintln!("Awaited barrier (1) {:?}", std::time::SystemTime::now());
            }
            eprintln!("limit finished {:?}", std::time::SystemTime::now());
        }
    };
    let live = {
        let engine = Arc::clone(&engine);
        let protocol = Arc::clone(&protocol);
        async move {
            engine.live().await;
            eprintln!("Live ended");
            let _ = protocol.send_message(Message::Ended).await;
            protocol.close().await;
            if !expired.load(core::sync::atomic::Ordering::Relaxed) {
                eprintln!("Awaiting barrier (2)");
                barrier.wait().await;
                eprintln!("Awaited barrier (2) {:?}", std::time::SystemTime::now());
            }
            eprintln!("live finished {:?}", std::time::SystemTime::now());
        }
    };
    let run = {
        let engine = Arc::clone(&engine);
        let protocol = Arc::clone(&protocol);
        let collection = Arc::clone(&collection);
        async move {
            let tracks_entry_outputs = Arc::new(AsyncRwLock::new(HashMap::new()));
            let tracks_entry_inputs = Arc::new(AsyncRwLock::new(HashMap::new()));
            loop {
                match {eprintln!("Awaiting msg");let msg = protocol.recv_message().await; eprintln!("Msg received: {msg:?}"); msg} {
                    Ok(Message::Instanciate(instanciate)) => {
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
                                            while let Ok(data) = input.recv_many().await {
                                                if protocol
                                                    .send_message(Message::OutputData(OutputData {
                                                        id: track_id,
                                                        name: name.clone(),
                                                        data: Into::<VecDeque<Value>>::into(data)
                                                            .into_iter()
                                                            .map(|val| val.into())
                                                            .collect(),
                                                    }))
                                                    .await
                                                    .is_err()
                                                {
                                                    input.close();
                                                    break;
                                                }
                                            }
                                            let _ = protocol
                                                .send_message(Message::CloseOutput(CloseOutput {
                                                    id: track_id,
                                                    name: name.clone(),
                                                }))
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
                    Ok(Message::InputData(input_data)) => {
                        if let Some(outputs) = tracks_entry_outputs.read().await.get(&input_data.id)
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
                    Ok(Message::CloseInput(close_input)) => {
                        if let Some(outputs) =
                            tracks_entry_outputs.read().await.get(&close_input.id)
                        {
                            if let Some(output) = outputs.get(&close_input.name) {
                                output.close().await;
                            }
                        }
                    }
                    Ok(Message::CloseOutput(close_output)) => {
                        if let Some(inputs) = tracks_entry_inputs.read().await.get(&close_output.id)
                        {
                            if let Some(input) = inputs.get(&close_output.name) {
                                input.close();
                                eprintln!("Input closed");
                            }
                        }
                    }
                    Ok(Message::Ended) => {
                        for (_, outputs) in tracks_entry_outputs.read().await.iter() {
                            for (_, output) in outputs {
                                output.close().await;
                            }
                        }
                        eprintln!("Ending engine (2)");
                        engine.end().await;
                        eprintln!("Ended engine (2) {:?}", std::time::SystemTime::now());
                        break;
                    }
                    Ok(Message::Probe) => {}
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Error in protocol: {err}");
                        eprintln!("Ending engine (3)");
                        engine.end().await;
                        eprintln!("Ended engine (3) {:?}", std::time::SystemTime::now());
                        break;
                    }
                }
                eprintln!("New loop");
            }
            protocol.close().await;
            eprintln!("run finished {:?}", std::time::SystemTime::now());
        }
    };
    let probe = {
        let engine = Arc::clone(&engine);
        let protocol = Arc::clone(&protocol);
        async move {
            loop {
                async_std::task::sleep(Duration::from_secs(10)).await;
                eprintln!("Probing {:?}", std::time::SystemTime::now());
                if let Err(err) = protocol.send_message(Message::Probe).await
                /*.is_err()*/
                {
                    eprintln!("Error when probing: {err}");
                    eprintln!("Ending engine (4)");
                    engine.end().await;
                    eprintln!("Ended engine (4)");
                    break;
                }
            }
            protocol.close().await;
            eprintln!("probe finished {:?}", std::time::SystemTime::now());
        }
    };

    futures::join!(limit, live, run, probe);
}

#[cfg(any(
    all(not(target_os = "windows"), not(target_vendor = "apple")),
    all(target_os = "windows", target_env = "gnu")
))]
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

#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
fn acceptor(
    certificate_chain: &[u8],
    key: &[u8],
) -> Result<TlsAcceptor, Box<dyn std::error::Error>> {
    let identity = native_tls::Identity::from_pkcs8(certificate_chain, key)?;
    let acceptor = native_tls::TlsAcceptor::new(identity)?;
    Ok(TlsAcceptor::from(acceptor))
}
