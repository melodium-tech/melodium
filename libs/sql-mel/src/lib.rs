#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::{common::executive::ResultStatus, *};
use melodium_macro::{check, mel_package, mel_model};
use async_std::sync::{RwLock as AsyncRwLock, Arc as AsyncArc};
use core::time::Duration;
use std::{collections::HashMap, sync::{Arc, Weak}};
use sqlx::{any::AnyPoolOptions, AnyPool};

#[derive(Debug)]
#[mel_model(
    param url string none
    param max_connections u32 10
    param min_connections u32 0
    param acquire_timeout u64 10000
    param idle_timeout Option<u64> 600000
    param max_lifetime Option<u64> 1800000
    source failure () () (
        failure Block<string>
    )
    initialize initialize
    continuous (continuous)
    shutdown shutdown
)]
pub struct SqlPool {
    model: Weak<SqlPoolModel>,
    pool: AsyncRwLock<Option<AnyPool>>,
    error: AsyncRwLock<Option<sqlx::Error>>,
}

impl SqlPool {
    fn new(model: Weak<SqlPoolModel>) -> Self {
        Self {
            model,
            pool: AsyncRwLock::new(None),
            error: AsyncRwLock::new(None),
        }
    }

    fn initialize(&self) {
        let model = self.model.upgrade().unwrap();

        match AnyPoolOptions::new().max_connections(model.get_max_connections())
        .min_connections(model.get_min_connections())
        .acquire_timeout(Duration::from_millis(model.get_acquire_timeout()))
        .idle_timeout(model.get_idle_timeout().map(|millis| Duration::from_millis(millis)))
        .max_lifetime(model.get_max_lifetime().map(|millis| Duration::from_millis(millis)))
        .connect_lazy(&model.get_url()) {
            Ok(pool) => async_std::task::block_on(async {
                *self.pool.write().await = Some(pool);
                }),
            Err(error) => {
                async_std::task::block_on(async {
                *self.error.write().await = Some(error);
                })
            },
        }
    }

    async fn continuous(&self) {
        if let Some(error) = self.error.read().await.as_ref() {
            let model = self.model.upgrade().unwrap();
            let error = error.to_string();
            model.new_failure(None, &HashMap::new(), Some(Box::new(move |mut outputs| {
                let failure = outputs.get("failure");
                vec![Box::new(Box::pin(async move {
                    let _ = failure.send_one(Value::String(error)).await;
                    failure.close().await;
                    ResultStatus::Ok
                }))]
            }))).await;
        }
    }

    fn shutdown(&self) {
        async_std::task::block_on(async {
            if let Some(pool) = self.pool.read().await.as_ref() {
                pool.close().await;
            }
        });
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    pub(crate) fn pool(&self) -> &AsyncRwLock<Option<AnyPool>> {
        &self.pool
    }
}

mel_package!();
