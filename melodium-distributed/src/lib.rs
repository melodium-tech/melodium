mod error;
mod messages;
mod protocol;

use std::{collections::VecDeque, sync::Arc};

pub use error::{DistributionError, DistributionResult};

use async_std::net::{SocketAddr, TcpListener};
use melodium_common::{
    descriptor::{Entry, Identifier, Model as CommonModel, Treatment as CommonTreatment, Version},
    executive::{ResultStatus, TransmissionValue, Value},
};
use melodium_engine::descriptor::{Model, Treatment};
use melodium_loader::Loader;
use melodium_sharing::{SharingError, SharingResult};
use messages::{ConfirmDistribution, InputData, Message, OutputData};
use protocol::Protocol;

static VERSION: Version = Version::new(0, 1, 0);

pub async fn launch_listen(bind: SocketAddr, version: &Version, loader: Loader) {
    let listener = TcpListener::bind(bind).await.unwrap();
    let (stream, _addr) = listener.accept().await.unwrap();

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
            engine
                .instanciate(Some(Box::new(move |entry_outputs, entry_inputs| {
                    let mut inputs_management = Vec::new();
                    for (name, input) in entry_inputs {
                        let protocol = Arc::clone(&protocol);
                        let listener = async move {
                            while let Ok(data) = input.recv_many().await {
                                if protocol
                                    .send_message(Message::OutputData(OutputData {
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
                        };
                        inputs_management.push(Box::new(Box::pin(listener)));
                    }

                    let protocol_management = async move {
                        loop {
                            match protocol.recv_message().await {
                                Ok(Message::InputData(InputData { name, data })) => {
                                    if let Some(output) = entry_outputs.get(&name) {
                                        let _ = output
                                            .send_many(TransmissionValue::Other(
                                                data.into_iter()
                                                    .map(|val| val.try_into().unwrap())
                                                    .collect::<VecDeque<Value>>(),
                                            ))
                                            .await;
                                    }
                                }
                                Ok(Message::Ended) => {
                                    for (_, output) in &entry_outputs {
                                        output.close().await;
                                    }
                                    break;
                                }
                                Err(_) => {
                                    break;
                                }
                                _ => {
                                    break;
                                }
                            }
                        }
                    };

                    vec![Box::new(Box::pin(async move {
                        futures::join!(
                            protocol_management,
                            futures::future::join_all(inputs_management)
                        );

                        ResultStatus::Ok
                    }))]
                })))
                .await;
        }
    };

    futures::join!(live, run);
}
