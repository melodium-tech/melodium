use crate::filesystem::*;
use async_std::fs::{self, DirBuilder, OpenOptions};
use async_std::stream::StreamExt;
use async_trait::async_trait;
use async_walkdir::{Filtering, WalkDir};
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_core::*;
use melodium_macro::{check, mel_function};
use std::path::{Path, PathBuf};
use std::{fmt::Debug, sync::Arc};

#[derive(Debug)]
struct LocalFileSystemEngine {}

#[async_trait]
impl FileSystemEngine for LocalFileSystemEngine {
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
    ) {
        let file = OpenOptions::new().read(true).open(path).await;
        match file {
            Ok(mut file) => {
                reached().await;
                reachedclose().await;
                let mut vec = vec![0; 2usize.pow(20)];
                let mut fail = false;
                loop {
                    match file.read(&mut vec).await {
                        Ok(n) if n > 0 => {
                            vec.truncate(n);
                            check!(data(vec.into()).await);
                            vec = vec![0; 2usize.pow(20)];
                        }
                        Ok(_) => {
                            break;
                        }
                        Err(err) => {
                            let _ = failed().await;
                            let _ = errors(err.to_string()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    let _ = completed().await;
                }
            }
            Err(err) => {
                let _ = failed().await;
                let _ = errors(err.to_string()).await;
            }
        }
        let _ = finished().await;
    }
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
    ) {
        let path = PathBuf::from(path);
        if let Err(err) = DirBuilder::new()
            .recursive(true)
            .create(path.parent().unwrap_or(Path::new("")))
            .await
        {
            failed().await;
            let _ = errors(err.to_string()).await;
            finished().await;
        } else {
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
                    while let Ok(data) = data().await {
                        match file.write_all(&data).await {
                            Ok(_) => {
                                written_amount += data.len() as u128;
                                let _ = amount(written_amount).await;
                            }
                            Err(err) => {
                                failed().await;
                                let _ = errors(err.to_string()).await;
                                fail = true;
                                break;
                            }
                        }
                    }
                    if !fail {
                        completed().await;
                    }
                }
                Err(err) => {
                    failed().await;
                    let _ = errors(err.to_string()).await;
                }
            }
            finished().await;
        }
    }
    async fn create_dir(
        &self,
        path: &str,
        recursive: bool,
        success: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
    ) {
        match if recursive {
            fs::create_dir_all(path).await
        } else {
            fs::create_dir(path).await
        } {
            Ok(()) => {
                success().await;
            }
            Err(err) => {
                error(err.to_string()).await;
                failed().await;
            }
        }
    }
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

        let mut success = true;
        while let Some(entry) = dir_entries.next().await {
            match entry {
                Ok(entry) => check!(entries(entry.path().to_string_lossy().to_string()).await),
                Err(err) => {
                    success = false;
                    let _ = errors(err.to_string()).await;
                }
            }
        }
        let _ = finished().await;
        if success {
            let _ = completed().await;
        } else {
            let _ = failed().await;
        }
    }
}

#[mel_function]
pub fn local_filesystem() -> Option<FileSystem> {
    Some(FileSystem {
        filesystem: Arc::new(LocalFileSystemEngine {}),
    })
}
