#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use async_std::net::{IpAddr, SocketAddr, TcpStream};
use async_std::sync::{Arc as AsyncArc, RwLock as AsyncRwLock};
use common::{
    descriptor::{Identifier, Version},
    executive::ResultStatus,
};
use core::str::FromStr;
use melodium_core::*;
use melodium_distributed::{AskDistribution, LoadAndLaunch, Message, Protocol};
use melodium_macro::{check, mel_model, mel_package, mel_treatment};
use melodium_sharing::Collection;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
};
use std_mel::data::*;

#[derive(Debug)]
#[mel_model(
    param host string none
    param port u16 none
    param treatment string none
    source ready () () (
        trigger Block<void>
    )
    source distribution_failure () () (
        failure Block<string>
    )
    continuous (continuous)
    shutdown shutdown
)]
pub struct DistributionEngine {
    model: Weak<DistributionEngineModel>,
    protocol: AsyncRwLock<Option<AsyncArc<Protocol<TcpStream>>>>,
}

impl DistributionEngine {
    fn new(model: Weak<DistributionEngineModel>) -> Self {
        Self {
            model,
            protocol: AsyncRwLock::new(None),
        }
    }

    pub async fn start(&self, params: HashMap<String, Value>) {
        let model = self.model.upgrade().unwrap();

        let entrypoint = match Identifier::from_str(&model.get_treatment()) {
            Ok(id) => id,
            Err(err) => {
                self.distribution_failure(format!("'{err}' is not a valid identifier"))
                    .await;
                return;
            }
        };

        let mut protocol_lock = self.protocol.write().await;

        if protocol_lock.is_none() {
            let addrs = match IpAddr::from_str(&model.get_host()) {
                Ok(addr) => SocketAddr::new(addr, model.get_port()),
                Err(err) => {
                    self.distribution_failure(err.to_string()).await;
                    return;
                }
            };

            let protocol = match TcpStream::connect(addrs).await {
                Ok(stream) => Protocol::new(stream),
                Err(err) => {
                    self.distribution_failure(err.to_string()).await;
                    return;
                }
            };

            match protocol
                .send_message(Message::AskDistribution(AskDistribution {
                    melodium_version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
                    distribution_version: melodium_distributed::VERSION.clone(),
                }))
                .await
            {
                Ok(_) => match protocol.recv_message().await {
                    Ok(Message::ConfirmDistribution(confirm)) => {
                        if !confirm.accept {
                            self.distribution_failure(format!("Cannot distribute, remote engine version is {} with protocol version {}, while local engine version is {} with protocol version {}.", confirm.melodium_version, confirm.distribution_version, env!("CARGO_PKG_VERSION"), melodium_distributed::VERSION)).await;
                            return;
                        }
                    }
                    Ok(_) => {
                        self.distribution_failure("Unexpected response message".to_string())
                            .await;
                        return;
                    }
                    Err(err) => {
                        self.distribution_failure(err.to_string()).await;
                        return;
                    }
                },
                Err(err) => {
                    self.distribution_failure(err.to_string()).await;
                    return;
                }
            }

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
                        melodium_distributed::LaunchStatus::Ok => {
                            *protocol_lock = Some(Arc::new(protocol));
                            model.new_ready(None, &HashMap::new(), None).await;
                        }
                        melodium_distributed::LaunchStatus::Failure(err) => {
                            self.distribution_failure(err.to_string()).await;
                            return;
                        }
                        _ => {
                            self.distribution_failure("Unexpected response message".to_string())
                                .await;
                            return;
                        }
                    },
                    Ok(_) => {
                        self.distribution_failure("Unexpected response message".to_string())
                            .await;
                        return;
                    }
                    Err(err) => {
                        self.distribution_failure(err.to_string()).await;
                        return;
                    }
                },
                Err(err) => {
                    self.distribution_failure(err.to_string()).await;
                    return;
                }
            }
        }
    }

    async fn continuous(&self) {
        let model = self.model.upgrade().unwrap();
    }

    fn shutdown(&self) {
        async_std::task::block_on(async move {
            if let Some(protocol) = (*self.protocol.read().await).as_ref().cloned() {
                let _ = protocol.send_message(Message::Ended).await;
            }
        });
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    async fn distribution_failure(&self, message: String) {
        self.model
            .upgrade()
            .unwrap()
            .new_distribution_failure(
                None,
                &HashMap::new(),
                Some(Box::new(move |mut outputs| {
                    let failure = outputs.get("failure");

                    vec![Box::new(Box::pin(async move {
                        let _ = failure.send_one(message.into()).await;
                        failure.close().await;
                        ResultStatus::Ok
                    }))]
                })),
            )
            .await;
    }
}

#[mel_treatment(
    model distributor DistributionEngine
)]
pub async fn start(params: Map) {
    let model = DistributionEngineModel::into(distributor);
    let distributor = model.inner();

    let params = params.map.clone();

    distributor.start(params).await;
}

#[mel_treatment(
    model distributor DistributionEngine
)]
pub async fn deport(params: Map) {}

mel_package!();
