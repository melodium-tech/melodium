use crate::error::DistributionResult;
use crate::protocol::Protocol;
use crate::{messages, messages::*, VERSION};
#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
use async_native_tls::TlsAcceptor;
use async_std::{
    net::{SocketAddr, TcpListener},
    sync::RwLock as AsyncRwLock,
};
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
use melodium_sharing::{SharingError, SharingResult};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

pub const INTERMEDIATE_CERTIFICATE: &[u8; 1678] = include_bytes!("../melodium-ica.der");
pub const LOCALHOST_CERTIFICATE: &[u8; 1722] = include_bytes!("../melodium-localhost.der");
pub const LOCALHOST_KEY: &[u8; 2376] = include_bytes!("../melodium-localhost.key.der");
pub const LOCALHOST_CHAIN: &[u8; 7757] = include_bytes!("../melodium-localhost.pfx");

pub async fn launch_listen(bind: SocketAddr, version: &Version, loader: Loader) {
    let listener = TcpListener::bind(bind).await.unwrap();
    let (stream, _addr) = listener.accept().await.unwrap();

    let acceptor = acceptor().await.unwrap();

    let stream = acceptor.accept(stream).await.unwrap();

    let protocol = Arc::new(Protocol::new(stream));

    match protocol.recv_message().await {
        Ok(Message::AskDistribution(ask)) => {
            let accept = &ask.melodium_version == version && ask.distribution_version == VERSION;
            protocol
                .send_message(Message::ConfirmDistribution(ConfirmDistribution {
                    melodium_version: version.clone(),
                    distribution_version: VERSION.clone(),
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

    if result.is_failure() {
        todo!()
    }

    let mut collection = loader.collection().clone();

    // Proceed descriptor build
    for element in distributed_collection.elements() {
        if !element.is_compiled() {
            match element {
                melodium_sharing::Element::Model(m) => {
                    let model: Option<Arc<Model>> = result.merge_degrade_failure(
                        DistributionResult::from(m.make_descriptor(&collection)),
                    );
                    if let Some(model) = model {
                        collection.insert(Entry::Model(Arc::clone(&model) as Arc<dyn CommonModel>));
                    }
                }
                melodium_sharing::Element::Treatment(t) => {
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
                melodium_sharing::Element::Model(m) => {
                    result = result
                        .and_degrade_failure(DistributionResult::from(m.make_design(&collection)));
                }
                melodium_sharing::Element::Treatment(t) => {
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
    let engine = melodium_engine::new_engine(collection);
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

    let live = {
        let engine = Arc::clone(&engine);
        let protocol = Arc::clone(&protocol);
        async move {
            engine.live().await;
            let _ = protocol.send_message(Message::Ended).await;
        }
    };
    let run = {
        let engine = Arc::clone(&engine);
        let protocol = Arc::clone(&protocol);
        async move {
            let tracks_entry_outputs = Arc::new(AsyncRwLock::new(HashMap::new()));
            let tracks_entry_inputs = Arc::new(AsyncRwLock::new(HashMap::new()));
            loop {
                match protocol.recv_message().await {
                    Ok(Message::Instanciate(instanciate)) => {
                        let protocol = Arc::clone(&protocol);
                        let tracks_entry_outputs = Arc::clone(&tracks_entry_outputs);
                        let tracks_entry_inputs = Arc::clone(&tracks_entry_inputs);
                        let track_id = instanciate.id;
                        engine
                            .instanciate(Some(Box::new(move |entry_outputs, entry_inputs| {
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
                            })))
                            .await
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
                                            .map(|val| val.try_into().unwrap())
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
                            }
                        }
                    }
                    Ok(Message::Ended) => {
                        for (_, outputs) in tracks_entry_outputs.read().await.iter() {
                            for (_, output) in outputs {
                                output.close().await;
                            }
                        }
                        engine.end();
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => {
                        engine.end();
                        break;
                    }
                }
            }
        }
    };

    futures::join!(live, run);
}

#[cfg(any(
    all(not(target_os = "windows"), not(target_vendor = "apple")),
    all(target_os = "windows", target_env = "gnu")
))]
async fn acceptor() -> Result<TlsAcceptor, Box<dyn std::error::Error>> {
    use futures_rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

    Ok(TlsAcceptor::from(Arc::new(
        futures_rustls::rustls::ServerConfig::builder_with_protocol_versions(&[
            &futures_rustls::rustls::version::TLS13,
        ])
        .with_no_client_auth()
        .with_single_cert(
            vec![
                CertificateDer::from(LOCALHOST_CERTIFICATE.as_slice()),
                CertificateDer::from(INTERMEDIATE_CERTIFICATE.as_slice()),
                CertificateDer::from(crate::ROOT_CERTIFICATE.as_slice()),
            ],
            futures_rustls::pki_types::PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                LOCALHOST_KEY.as_slice(),
            )),
        )?,
    )))
}

#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
async fn acceptor() -> Result<TlsAcceptor, Box<dyn std::error::Error>> {
    Ok(TlsAcceptor::new(LOCALHOST_CHAIN.as_slice(), "lyoko").await?)
}
