mod error;
mod messages;
mod protocol;

use std::sync::Arc;

pub use error::{DistributionError, DistributionResult};

use async_std::net::{SocketAddr, TcpListener};
use melodium_common::descriptor::{
    Entry, Identifier, Model as CommonModel, Treatment as CommonTreatment, Version,
};
use melodium_engine::descriptor::{Model, Treatment};
use melodium_loader::Loader;
use messages::{ConfirmDistribution, Message};
use protocol::Protocol;

static VERSION: Version = Version::new(0, 1, 0);

pub async fn launch_listen(bind: SocketAddr, version: &Version, loader: Loader) {
    let listener = TcpListener::bind(bind).await.unwrap();
    let (stream, _addr) = listener.accept().await.unwrap();

    let protocol = Protocol::new(stream);

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
                todo!()
            }
        }
    }

    if result.is_failure() {
        todo!()
    }

    let mut collection = loader.collection().clone();

    // Proceed to design of the rest
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

    // Design
    // m.make_design(collection)

    // Give it to engine

    // Manage engine calls to entrypoint
}
