use async_trait::async_trait;
use core::{future::Future, pin::Pin};
use melodium_core::*;
use melodium_macro::mel_data;
use std::{fmt::Debug, sync::Arc};

pub type OnceTriggerCall<'a> =
    Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + Sync + 'a>;
pub type OnceMessageCall<'a> =
    Box<dyn FnOnce(String) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + Sync + 'a>;
pub type OutU128Call<'a> = Box<
    dyn Fn(u128) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>> + Send + Sync + 'a,
>;
pub type OutMessageCall<'a> = Box<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>> + Send + Sync + 'a,
>;
pub type InDataCall<'a> = Box<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<Vec<u8>, ()>> + Send + 'a>> + Send + Sync + 'a,
>;
pub type OutDataCall<'a> = Box<
    dyn Fn(VecDeque<u8>) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>>
        + Send
        + Sync
        + 'a,
>;

#[async_trait]
pub trait FileSystemEngine: Debug + Send + Sync {
    async fn read_file(
        &self,
        path: &str,
        data: OutDataCall<'async_trait>,
        reached: OnceTriggerCall<'async_trait>,
        reachedclose: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        errors: OutMessageCall<'async_trait>,
    );
    async fn write_file(
        &self,
        path: &str,
        append: bool,
        create: bool,
        new: bool,
        data: InDataCall<'async_trait>,
        amount: OutU128Call<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        errors: OutMessageCall<'async_trait>,
    );
    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
    );
    async fn scan_dir(
        &self,
        path: &str,
        recursive: bool,
        follow_links: bool,
        entries: OutMessageCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        errors: OutMessageCall<'async_trait>,
    );
}

#[derive(Debug, Serialize)]
#[mel_data]
pub struct FileSystem {
    #[serde(skip)]
    pub filesystem: Arc<dyn FileSystemEngine>,
}
