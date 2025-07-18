#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "mock", allow(unused))]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

use async_std::stream::StreamExt;
use async_std::sync::{Arc as AsyncArc, RwLock as AsyncRwLock};
use core::time::Duration;
use melodium_core::{common::executive::ResultStatus, *};
use melodium_macro::{check, mel_model, mel_package, mel_treatment};
use sqlx::any::{AnyArguments, AnyRow, AnyTypeInfoKind};
use sqlx::query::Query;
use sqlx::Any;
use sqlx::{any::AnyPoolOptions, AnyPool, Column, QueryBuilder, Row};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use std_mel::data::map::*;

fn postgres_bind_replace(mut sql_to_bind: String, bind_symbol: &str) -> String {
    let bind_num = sql_to_bind.matches(bind_symbol).count();

    for i in 1..=bind_num {
        sql_to_bind = sql_to_bind
            .replacen(bind_symbol, &format!("${i}"), 1)
            .to_string();
    }

    sql_to_bind
}

fn bind_value<'q>(
    query: Query<'q, Any, AnyArguments<'q>>,
    value: &Value,
) -> Query<'q, Any, AnyArguments<'q>> {
    match value {
        Value::Void(_) => query.bind(None::<bool>),
        Value::I8(n) => query.bind(*n as i16),
        Value::I16(n) => query.bind(*n),
        Value::I32(n) => query.bind(*n as i32),
        Value::I64(n) => query.bind(*n as i64),
        Value::I128(n) => query.bind(*n as f64),
        Value::U8(n) => query.bind(*n as i16),
        Value::U16(n) => query.bind(*n as i32),
        Value::U32(n) => query.bind(*n as i64),
        Value::U64(n) => query.bind(*n as f64),
        Value::U128(n) => query.bind(*n as f64),
        Value::F32(n) => query.bind(*n),
        Value::F64(n) => query.bind(*n),
        Value::Bool(b) => query.bind(*b),
        Value::Byte(n) => query.bind(vec![*n]),
        Value::Char(c) => query.bind(c.to_string()),
        Value::String(s) => query.bind(s.clone()),
        Value::Vec(_) => query.bind(None::<bool>),
        Value::Option(o) => match o {
            None => query.bind(None::<bool>),
            Some(v) => bind_value(query, v),
        },
        Value::Data(d) => {
            if value
                .datatype()
                .implements(&melodium_core::common::descriptor::DataTrait::ToString)
            {
                query.bind(d.to_string())
            } else {
                query.bind(None::<bool>)
            }
        }
    }
}

fn get_row_as_map(row: &AnyRow) -> Map {
    let mut map = HashMap::with_capacity(row.len());
    for column in row.columns() {
        map.insert(
            column.name().to_string(),
            match column.type_info().kind() {
                AnyTypeInfoKind::Null => Value::Option(None),
                AnyTypeInfoKind::Bool => row
                    .try_get::<bool, _>(column.ordinal())
                    .map(|b| Value::Bool(b))
                    .unwrap_or_else(|_| Value::Option(None)),
                AnyTypeInfoKind::SmallInt => row
                    .try_get::<i16, _>(column.ordinal())
                    .map(|n| Value::I16(n))
                    .unwrap_or_else(|_| Value::Option(None)),
                AnyTypeInfoKind::Integer => row
                    .try_get::<i32, _>(column.ordinal())
                    .map(|n| Value::I32(n))
                    .unwrap_or_else(|_| Value::Option(None)),
                AnyTypeInfoKind::BigInt => row
                    .try_get::<i64, _>(column.ordinal())
                    .map(|n| Value::I64(n))
                    .unwrap_or_else(|_| Value::Option(None)),
                AnyTypeInfoKind::Real => row
                    .try_get::<f32, _>(column.ordinal())
                    .map(|n| Value::F32(n))
                    .unwrap_or_else(|_| Value::Option(None)),
                AnyTypeInfoKind::Double => row
                    .try_get::<f64, _>(column.ordinal())
                    .map(|n| Value::F64(n))
                    .unwrap_or_else(|_| Value::Option(None)),
                AnyTypeInfoKind::Text => row
                    .try_get::<String, _>(column.ordinal())
                    .map(|s| Value::String(s))
                    .unwrap_or_else(|_| Value::Option(None)),
                AnyTypeInfoKind::Blob => row
                    .try_get::<Vec<u8>, _>(column.ordinal())
                    .map(|d| Value::Vec(d.into_iter().map(|v| Value::Byte(v)).collect()))
                    .unwrap_or_else(|_| Value::Option(None)),
            },
        );
    }
    Map::new_with(map)
}

#[derive(Debug)]
#[mel_model(
    param url string none
    param max_connections u32 10
    param min_connections u32 0
    param acquire_timeout u64 10000
    param idle_timeout Option<u64> 600000
    param max_lifetime Option<u64> 1800000
    source connected () () (
        trigger Block<void>
    )
    source failure () () (
        failed Block<void>
        error Block<string>
    )
    source closed () () (
        trigger Block<void>
    )
    initialize initialize
    shutdown shutdown
)]
pub struct SqlPool {
    model: Weak<SqlPoolModel>,
    pool: AsyncRwLock<Option<AsyncArc<AnyPool>>>,
}

impl SqlPool {
    fn new(model: Weak<SqlPoolModel>) -> Self {
        Self {
            model,
            pool: AsyncRwLock::new(None),
        }
    }

    fn initialize(&self) {
        sqlx::any::install_default_drivers();
    }

    pub async fn connect(&self) {
        let model = self.model.upgrade().unwrap();

        let mut pool_lock = self.pool.write().await;
        if pool_lock.is_none() {
            match AnyPoolOptions::new()
                .max_connections(model.get_max_connections())
                .min_connections(model.get_min_connections())
                .acquire_timeout(Duration::from_millis(model.get_acquire_timeout()))
                .idle_timeout(
                    model
                        .get_idle_timeout()
                        .map(|millis| Duration::from_millis(millis)),
                )
                .max_lifetime(
                    model
                        .get_max_lifetime()
                        .map(|millis| Duration::from_millis(millis)),
                )
                .connect_lazy(&model.get_url())
            {
                Ok(pool) => {
                    *pool_lock = Some(AsyncArc::new(pool));
                    model
                        .new_connected(
                            None,
                            &HashMap::new(),
                            Some(Box::new(move |mut outputs| {
                                let trigger = outputs.get("trigger");
                                vec![Box::new(Box::pin(async move {
                                    let _ = trigger.send_one(().into()).await;
                                    trigger.close().await;
                                    ResultStatus::Ok
                                }))]
                            })),
                        )
                        .await;
                }
                Err(error) => {
                    let err = error.to_string();
                    model
                        .new_failure(
                            None,
                            &HashMap::new(),
                            Some(Box::new(move |mut outputs| {
                                let failed = outputs.get("failed");
                                let error = outputs.get("error");
                                vec![Box::new(Box::pin(async move {
                                    let _ = failed.send_one(().into()).await;
                                    let _ = error.send_one(Value::String(err)).await;
                                    failed.close().await;
                                    error.close().await;
                                    ResultStatus::Ok
                                }))]
                            })),
                        )
                        .await;
                }
            }
        }
    }

    pub async fn close(&self) {
        let model = self.model.upgrade().unwrap();

        let mut pool_lock = self.pool.write().await;
        if let Some(pool) = pool_lock.as_ref() {
            pool.close().await;

            model
                .new_closed(
                    None,
                    &HashMap::new(),
                    Some(Box::new(move |mut outputs| {
                        let trigger = outputs.get("trigger");
                        vec![Box::new(Box::pin(async move {
                            let _ = trigger.send_one(().into()).await;
                            trigger.close().await;
                            ResultStatus::Ok
                        }))]
                    })),
                )
                .await;
        }
        *pool_lock = None;
    }

    fn shutdown(&self) {
        #[cfg(feature = "real")]
        async_std::task::block_on(async {
            if let Some(pool) = self.pool.read().await.as_ref() {
                pool.close().await;
            }
        });
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    pub(crate) async fn pool(&self) -> Result<AsyncArc<AnyPool>, sqlx::Error> {
        match self.pool.read().await.as_ref() {
            Some(pool) => Ok(AsyncArc::clone(pool)),
            None => Err(sqlx::Error::PoolClosed),
        }
    }
}

#[mel_treatment(
    model sql_pool SqlPool
    input trigger Block<void>
)]
pub async fn connect() {
    let model = SqlPoolModel::into(sql_pool);
    let sql_pool = model.inner();

    if let Ok(_) = trigger.recv_one().await {
        sql_pool.connect().await;
    }
}

#[mel_treatment(
    model sql_pool SqlPool
    input trigger Block<void>
)]
pub async fn close() {
    let model = SqlPoolModel::into(sql_pool);
    let sql_pool = model.inner();

    if let Ok(_) = trigger.recv_one().await {
        sql_pool.close().await;
    }
}

#[mel_treatment(
    input trigger Block<void>
    output affected Block<u64>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    model sql_pool SqlPool
)]
pub async fn execute_raw(sql: string) {
    match SqlPoolModel::into(sql_pool).inner().pool().await {
        Ok(pool) => match sqlx::raw_sql(&sql).execute(&*pool).await {
            Ok(result) => {
                let _ = completed.send_one(().into()).await;
                let _ = affected.send_one(Value::U64(result.rows_affected())).await;
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(err.to_string().into()).await;
            }
        },
        Err(err) => {
            let _ = failed.send_one(().into()).await;
            let _ = error.send_one(err.to_string().into()).await;
        }
    }
    let _ = finished.send_one(().into()).await;
}

#[mel_treatment(
    input bind Block<Map>
    output affected Block<u64>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    default bind_symbol "?"
    model sql_pool SqlPool
)]
pub async fn execute(sql: string, bindings: Vec<string>, bind_symbol: string) {
    if let Ok(bind) = bind.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Map>()
            .unwrap()
    }) {
        match SqlPoolModel::into(sql_pool).inner().pool().await {
            Ok(pool) => {
                let sql = match pool.connect_options().database_url.scheme() {
                    "postgres" => postgres_bind_replace(sql, &bind_symbol),
                    _ => sql,
                };
                let mut query = sqlx::query(&sql);

                for binding in &bindings {
                    if let Some(val) = bind.map.get(binding) {
                        query = bind_value(query, val);
                    } else {
                        query = query.bind(None::<bool>);
                    }
                }

                match query.execute(&*pool).await {
                    Ok(result) => {
                        let _ = completed.send_one(().into()).await;
                        let _ = affected.send_one(Value::U64(result.rows_affected())).await;
                    }
                    Err(err) => {
                        let _ = failed.send_one(().into()).await;
                        let _ = error.send_one(err.to_string().into()).await;
                    }
                }
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(err.to_string().into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
}

#[mel_treatment(
    input bind Stream<Map>
    output affected Stream<u64>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output errors Stream<string>
    default bind_symbol "?"
    default stop_on_failure true
    model sql_pool SqlPool
)]
pub async fn execute_each(
    sql: string,
    bindings: Vec<string>,
    bind_symbol: string,
    stop_on_failure: bool,
) {
    match SqlPoolModel::into(sql_pool).inner().pool().await {
        Ok(pool) => {
            let mut success = true;
            while let Ok(bind) = bind.recv_one().await.map(|val| {
                GetData::<Arc<dyn Data>>::try_data(val)
                    .unwrap()
                    .downcast_arc::<Map>()
                    .unwrap()
            }) {
                let sql = match pool.connect_options().database_url.scheme() {
                    "postgres" => postgres_bind_replace(sql.clone(), &bind_symbol),
                    _ => sql.clone(),
                };
                let mut query = sqlx::query(&sql);

                for binding in &bindings {
                    if let Some(val) = bind.map.get(binding) {
                        query = bind_value(query, val);
                    } else {
                        query = query.bind(None::<bool>);
                    }
                }

                match query.execute(&*pool).await {
                    Ok(result) => {
                        let _ = affected.send_one(Value::U64(result.rows_affected())).await;
                    }
                    Err(error) => {
                        success = false;
                        let _ = errors.send_one(error.to_string().into()).await;
                        if stop_on_failure {
                            break;
                        }
                    }
                }
            }
            if success {
                let _ = completed.send_one(().into()).await;
            } else {
                let _ = failed.send_one(().into()).await;
            }
            let _ = finished.send_one(().into()).await;
        }
        Err(error) => {
            let _ = failed.send_one(().into()).await;
            let _ = errors.send_one(error.to_string().into()).await;
            let _ = finished.send_one(().into()).await;
        }
    }
}

#[mel_treatment(
    default separator ", "
    default stop_on_failure true
    default bind_limit 65535
    default bind_symbol "?"
    input bind Stream<Map>
    output affected Stream<u64>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output errors Stream<string>
    model sql_pool SqlPool
)]
pub async fn execute_batch(
    base: string,
    batch: string,
    bindings: Vec<string>,
    bind_symbol: string,
    bind_limit: u64,
    separator: string,
    stop_on_failure: bool,
) {
    let limit = bind_limit.min(65535);
    let batch_max = limit / bindings.len() as u64;

    match SqlPoolModel::into(sql_pool).inner().pool().await {
        Ok(pool) => {
            let mut success = true;
            'main: loop {
                let mut query_builder = QueryBuilder::new(base.as_str());

                let mut full_batch = Vec::with_capacity(batch_max as usize);
                for _ in 0..batch_max {
                    if let Ok(bind) = bind.recv_one().await.map(|val| {
                        GetData::<Arc<dyn Data>>::try_data(val)
                            .unwrap()
                            .downcast_arc::<Map>()
                            .unwrap()
                    }) {
                        full_batch.push(bind);
                    } else {
                        break;
                    }
                }

                if full_batch.is_empty() {
                    break;
                }

                let mut query = query_builder
                    .push({
                        let batch = std::iter::repeat(batch.as_str())
                            .take(full_batch.len())
                            .collect::<Vec<_>>()
                            .join(&separator);
                        match pool.connect_options().database_url.scheme() {
                            "postgres" => postgres_bind_replace(batch, &bind_symbol),
                            _ => batch,
                        }
                    })
                    .build();

                for b in full_batch {
                    for binding in &bindings {
                        if let Some(val) = b.map.get(binding) {
                            query = bind_value(query, val);
                        } else {
                            query = query.bind(None::<bool>);
                        }
                    }
                }

                match query.execute(&*pool).await {
                    Ok(result) => {
                        let _ = affected.send_one(Value::U64(result.rows_affected())).await;
                    }
                    Err(error) => {
                        success = false;
                        let _ = errors.send_one(error.to_string().into()).await;
                        if stop_on_failure {
                            break 'main;
                        }
                    }
                }
            }
            if success {
                let _ = completed.send_one(().into()).await;
            } else {
                let _ = failed.send_one(().into()).await;
            }
            let _ = finished.send_one(().into()).await;
        }
        Err(error) => {
            let _ = failed.send_one(().into()).await;
            let _ = errors.send_one(error.to_string().into()).await;
            let _ = finished.send_one(().into()).await;
        }
    }
}

#[mel_treatment(
    input bind Block<Map>
    output data Stream<Map>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output errors Stream<string>
    default bind_symbol "?"
    model sql_pool SqlPool
)]
pub async fn fetch(sql: string, bindings: Vec<string>, bind_symbol: string) {
    if let Ok(bind) = bind.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Map>()
            .unwrap()
    }) {
        match SqlPoolModel::into(sql_pool).inner().pool().await {
            Ok(pool) => {
                let sql = match pool.connect_options().database_url.scheme() {
                    "postgres" => postgres_bind_replace(sql, &bind_symbol),
                    _ => sql,
                };
                let mut query = sqlx::query(&sql);

                for binding in &bindings {
                    if let Some(val) = bind.map.get(binding) {
                        query = bind_value(query, val);
                    } else {
                        query = query.bind(None::<bool>);
                    }
                }

                let mut stream = query.fetch(&*pool);
                let mut success = true;
                while let Some(row) = stream.next().await {
                    match row {
                        Ok(row) => {
                            let map = get_row_as_map(&row);
                            check!(
                                data.send_one(Value::Data(Arc::new(map) as Arc<dyn Data>))
                                    .await
                            )
                        }
                        Err(error) => {
                            success = false;
                            let _ = errors.send_one(error.to_string().into()).await;
                            break;
                        }
                    }
                }
                if success {
                    let _ = completed.send_one(().into()).await;
                } else {
                    let _ = failed.send_one(().into()).await;
                }
            }
            Err(error) => {
                let _ = failed.send_one(().into()).await;
                let _ = errors.send_one(error.to_string().into()).await;
            }
        }
        let _ = finished.send_one(().into()).await;
    }
}

#[mel_treatment(
    default separator ", "
    default stop_on_failure true
    default bind_limit 65535
    default bind_symbol "?"
    input bind Stream<Map>
    output data Stream<Map>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output errors Stream<string>
    model sql_pool SqlPool
)]
pub async fn fetch_batch(
    base: string,
    batch: string,
    bindings: Vec<string>,
    bind_limit: u64,
    bind_symbol: string,
    separator: string,
    stop_on_failure: bool,
) {
    let limit = bind_limit.min(65535);
    let batch_max = limit / bindings.len() as u64;

    match SqlPoolModel::into(sql_pool).inner().pool().await {
        Ok(pool) => {
            let mut success = true;
            'main: loop {
                let mut query_builder = QueryBuilder::new(base.as_str());

                let mut full_batch = Vec::with_capacity(batch_max as usize);
                for _ in 0..batch_max {
                    if let Ok(bind) = bind.recv_one().await.map(|val| {
                        GetData::<Arc<dyn Data>>::try_data(val)
                            .unwrap()
                            .downcast_arc::<Map>()
                            .unwrap()
                    }) {
                        full_batch.push(bind);
                    } else {
                        break;
                    }
                }

                if full_batch.is_empty() {
                    break;
                }

                let mut query = query_builder
                    .push({
                        let batch = std::iter::repeat(batch.as_str())
                            .take(full_batch.len())
                            .collect::<Vec<_>>()
                            .join(&separator);
                        match pool.connect_options().database_url.scheme() {
                            "postgres" => postgres_bind_replace(batch, &bind_symbol),
                            _ => batch,
                        }
                    })
                    .build();

                for b in full_batch {
                    for binding in &bindings {
                        if let Some(val) = b.map.get(binding) {
                            query = bind_value(query, val);
                        } else {
                            query = query.bind(None::<bool>);
                        }
                    }
                }

                let mut stream = query.fetch(&*pool);
                'result: while let Some(row) = stream.next().await {
                    match row {
                        Ok(row) => {
                            let map = get_row_as_map(&row);

                            let _ = data
                                .send_one(Value::Data(Arc::new(map) as Arc<dyn Data>))
                                .await;
                        }
                        Err(error) => {
                            success = false;
                            let _ = errors.send_one(error.to_string().into()).await;
                            if stop_on_failure {
                                break 'main;
                            } else {
                                break 'result;
                            }
                        }
                    }
                }
            }
            if success {
                let _ = completed.send_one(().into()).await;
            } else {
                let _ = failed.send_one(().into()).await;
            }
            let _ = finished.send_one(().into()).await;
        }
        Err(error) => {
            let _ = failed.send_one(().into()).await;
            let _ = errors.send_one(error.to_string().into()).await;
            let _ = finished.send_one(().into()).await;
        }
    }
}

mel_package!();
