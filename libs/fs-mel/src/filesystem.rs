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
        completed: &Box<dyn Output>,
        failed: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        errors: &Box<dyn Output>,
    );
    async fn write_file(
        &self,
        path: &str,
        append: bool,
        create: bool,
        new: bool,
        data: &Box<dyn Input>,
        amount: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failed: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        errors: &Box<dyn Output>,
    );
    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: &Box<dyn Output>,
        failed: &Box<dyn Output>,
        error: &Box<dyn Output>,
    );
    async fn scan_dir(
        &self,
        path: &str,
        recursive: bool,
        follow_links: bool,
        entries: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failed: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        errors: &Box<dyn Output>,
    );
}

#[derive(Debug, Serialize)]
#[mel_data]
pub struct FileSystem {
    #[serde(skip)]
    pub filesystem: Arc<dyn FileSystemEngine>,
}
