use crate::filesystem::*;
use async_std::fs::{self, OpenOptions};
use async_std::stream::StreamExt;
use async_trait::async_trait;
use async_walkdir::{Filtering, WalkDir};
use common::executive::{Input, Output};
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_core::*;
use melodium_macro::{check, mel_function};
use std::{fmt::Debug, sync::Arc};

#[derive(Debug)]
struct LocalFileSystemEngine {}

#[async_trait]
impl FileSystemEngine for LocalFileSystemEngine {
    async fn read_file(
        &self,
        path: &str,
        data: &Box<dyn Output>,
        reached: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        error: &Box<dyn Output>,
    ) {
        let file = OpenOptions::new().read(true).open(path).await;
        match file {
            Ok(mut file) => {
                let _ = reached.send_one(().into()).await;
                reached.close().await;
                let mut vec = vec![0; 2usize.pow(20)];
                let mut fail = false;
                loop {
                    match file.read(&mut vec).await {
                        Ok(n) if n > 0 => {
                            vec.truncate(n);
                            check!(data.send_many(TransmissionValue::Byte(vec.into())).await);
                            vec = vec![0; 2usize.pow(20)];
                        }
                        Ok(_) => {
                            break;
                        }
                        Err(err) => {
                            let _ = failure.send_one(().into()).await;
                            let _ = error.send_one(err.to_string().into()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    let _ = finished.send_one(().into()).await;
                }
            }
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = error.send_one(err.to_string().into()).await;
            }
        }
    }
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
    ) {
        let file = OpenOptions::new()
            .write(true)
            .append(append)
            .create(create)
            .create_new(new)
            .open(path)
            .await;
        match file {
            Ok(mut file) => {
                let mut written_amount = 0u128;
                let mut fail = false;
                while let Ok(data) = data
                    .recv_many()
                    .await
                    .map(|values| TryInto::<Vec<u8>>::try_into(values).unwrap())
                {
                    match file.write_all(&data).await {
                        Ok(_) => {
                            written_amount += data.len() as u128;
                            let _ = amount.send_one(written_amount.into()).await;
                        }
                        Err(err) => {
                            let _ = failure.send_one(().into()).await;
                            let _ = error.send_one(err.to_string().into()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    let _ = finished.send_one(().into()).await;
                }
            }
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = error.send_one(err.to_string().into()).await;
            }
        }
    }
    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        error: &Box<dyn Output>,
    ) {
        match if recursive {
            fs::create_dir_all(path).await
        } else {
            fs::create_dir(path).await
        } {
            Ok(()) => {
                let _ = success.send_one(().into()).await;
            }
            Err(err) => {
                let _ = error.send_one(err.to_string().into()).await;
                let _ = failure.send_one(().into()).await;
            }
        }
    }
    async fn scan_dir(
        &self,
        path: &str,
        recursive: bool,
        follow_links: bool,
        entries: &Box<dyn Output>,
        success: &Box<dyn Output>,
        error: &Box<dyn Output>,
    ) {
        let mut dir_entries = WalkDir::new(path).filter(move |entry| async move {
            match entry.file_type().await {
                Ok(file_type) => {
                    if file_type.is_dir() {
                        if recursive {
                            Filtering::Continue
                        } else {
                            Filtering::IgnoreDir
                        }
                    } else if file_type.is_symlink() {
                        if follow_links {
                            Filtering::Continue
                        } else {
                            Filtering::IgnoreDir
                        }
                    } else {
                        Filtering::Continue
                    }
                }
                Err(_) => Filtering::Continue,
            }
        });

        while let Some(entry) = dir_entries.next().await {
            match entry {
                Ok(entry) => check!(
                    entries
                        .send_one(entry.path().to_string_lossy().to_string().into())
                        .await
                ),
                Err(err) => {
                    let _ = error.send_one(err.to_string().into()).await;
                }
            }
        }
        let _ = success.send_one(().into()).await;
    }
}

#[mel_function]
pub fn local_filesystem() -> Option<FileSystem> {
    Some(FileSystem {
        filesystem: Arc::new(LocalFileSystemEngine {}),
    })
}
