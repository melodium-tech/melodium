use async_trait::async_trait;
use common::executive::{Input, Output};
use melodium_core::*;
use melodium_macro::mel_data;
use std::{fmt::Debug, sync::Arc};

#[async_trait]
pub trait FileSystemEngine: Debug + Send + Sync {
    async fn read_file(
        &self,
        path: &str,
        data: &Box<dyn Output>,
        reached: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        error: &Box<dyn Output>,
    );
    async fn write_file(
        &self,
        path: &str,
        append: bool,
        create: bool,
        new: bool,
        data: &Box<dyn Input>,
        amount: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        error: &Box<dyn Output>,
    );
    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        error: &Box<dyn Output>,
    );
    async fn scan_dir(
        &self,
        path: &str,
        recursive: bool,
        follow_links: bool,
        entries: &Box<dyn Output>,
        success: &Box<dyn Output>,
        error: &Box<dyn Output>,
    );
}

#[derive(Debug, Serialize)]
#[mel_data]
pub struct FileSystem {
    #[serde(skip)]
    pub filesystem: Arc<dyn FileSystemEngine>,
}
