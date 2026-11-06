use crate::filesystem::*;
use async_trait::async_trait;
use async_walkdir::{Filtering, WalkDir};
use futures::StreamExt;
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_core::*;
use melodium_macro::{check, mel_function};
use std::path::{Path, PathBuf};
use std::{fmt::Debug, sync::Arc};

/// Local file system, without root: paths are those of the process, absolute or relative to
/// its working directory.
#[derive(Debug)]
struct LocalFileSystemEngine;

impl LocalFileSystemEngine {
    async fn full_path(&self, path: &Path) -> std::io::Result<PathBuf> {
        Ok(path.into())
    }
}

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
        #[cfg(feature = "real")]
        {
            let path = match self.full_path(&PathBuf::from(path)).await {
                Ok(path) => path,
                Err(err) => {
                    failed().await;
                    let _ = errors(err.to_string()).await;
                    finished().await;
                    return;
                }
            };

            let file = async_fs::OpenOptions::new().read(true).open(path).await;
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
        #[cfg(feature = "mock")]
        {
            let _ = failed().await;
            let _ = errors("Mock mode".to_string()).await;
            let _ = finished().await;
        }
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
        #[cfg(feature = "real")]
        {
            let path = match self.full_path(&PathBuf::from(path)).await {
                Ok(path) => path,
                Err(err) => {
                    failed().await;
                    let _ = errors(err.to_string()).await;
                    finished().await;
                    return;
                }
            };

            if let Err(err) = async_fs::DirBuilder::new()
                .recursive(true)
                .create(path.parent().unwrap_or(Path::new("")))
                .await
            {
                failed().await;
                let _ = errors(err.to_string()).await;
                finished().await;
            } else {
                let file = async_fs::OpenOptions::new()
                    .write(true)
                    .append(append)
                    .create(create)
                    .create_new(new)
                    .open(path)
                    .await;
                match file {
                    Ok(mut file) => {
                        let mut written_amount = 0u128;
                        let mut error = None;
                        while let Ok(data) = data().await {
                            match file.write_all(&data).await {
                                Ok(_) => {
                                    written_amount += data.len() as u128;
                                    let _ = amount(written_amount).await;
                                }
                                Err(err) => {
                                    error = Some(err);
                                    break;
                                }
                            }
                        }
                        // Writes run in the background: flushing waits for them, and gives their errors.
                        if error.is_none() {
                            error = file.flush().await.err();
                        }
                        match error {
                            None => completed().await,
                            Some(err) => {
                                failed().await;
                                let _ = errors(err.to_string()).await;
                            }
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
        #[cfg(feature = "mock")]
        {
            failed().await;
            let _ = errors("Mock mode".to_string()).await;
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
        #[cfg(feature = "real")]
        {
            let path = match self.full_path(&PathBuf::from(path)).await {
                Ok(path) => path,
                Err(err) => {
                    failed().await;
                    error(err.to_string()).await;
                    return;
                }
            };

            match if recursive {
                async_fs::create_dir_all(path).await
            } else {
                async_fs::create_dir(path).await
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
        #[cfg(feature = "mock")]
        {
            error("Mock mode".to_string()).await;
            failed().await;
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
        let path = match self.full_path(&PathBuf::from(path)).await {
            Ok(path) => path,
            Err(err) => {
                failed().await;
                let _ = errors(err.to_string()).await;
                return;
            }
        };

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
    async fn metadata(&self, path: &str) -> Result<Metadata, String> {
        let path = PathBuf::from(path);
        local(move || {
            let metadata = std::fs::symlink_metadata(&path)?;
            let file_type = metadata.file_type();
            Ok(Metadata {
                kind: if file_type.is_symlink() {
                    EntryKind::Link
                } else if file_type.is_dir() {
                    EntryKind::Directory
                } else if file_type.is_file() {
                    EntryKind::File
                } else {
                    EntryKind::Other
                },
                size: metadata.len(),
                mode: mode(&metadata.permissions()),
                modified: metadata.modified().ok(),
            })
        })
        .await
    }

    async fn read_dir(&self, path: &str) -> Result<Vec<String>, String> {
        let path = PathBuf::from(if path.is_empty() { "." } else { path });
        local(move || {
            std::fs::read_dir(&path)?
                .map(|entry| {
                    entry.and_then(|entry| {
                        entry.file_name().into_string().map_err(|name| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("{} is not a valid name", name.to_string_lossy()),
                            )
                        })
                    })
                })
                .collect()
        })
        .await
    }

    async fn read_link(&self, path: &str) -> Result<String, String> {
        let path = PathBuf::from(path);
        local(move || std::fs::read_link(&path))
            .await?
            .into_os_string()
            .into_string()
            .map_err(|target| format!("{} is not a valid target", target.to_string_lossy()))
    }

    async fn create_link(&self, target: &str, path: &str) -> Result<(), String> {
        #[cfg(unix)]
        {
            let target = PathBuf::from(target);
            let path = PathBuf::from(path);
            local(move || std::os::unix::fs::symlink(&target, &path)).await
        }
        #[cfg(not(unix))]
        {
            let _ = (target, path);
            Err("links can only be created on Unix systems".to_string())
        }
    }

    async fn set_metadata(
        &self,
        path: &str,
        mode: Option<u32>,
        modified: Option<std::time::SystemTime>,
    ) -> Result<(), String> {
        let path = PathBuf::from(path);
        local(move || {
            if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
                return Ok(());
            }
            // Times first, as the mode could forbid opening the entry.
            if let Some(modified) = modified {
                std::fs::File::open(&path)?.set_modified(modified)?;
            }
            if let Some(mode) = mode {
                set_mode(&path, mode)?;
            }
            Ok(())
        })
        .await
    }

    async fn remove(&self, path: &str, recursive: bool) -> Result<(), String> {
        let path = PathBuf::from(path);
        local(move || {
            if std::fs::symlink_metadata(&path)?.is_dir() {
                if recursive {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_dir(&path)
                }
            } else {
                std::fs::remove_file(&path)
            }
        })
        .await
    }

    async fn rename(&self, from: &str, to: &str) -> Result<(), String> {
        let from = PathBuf::from(from);
        let to = PathBuf::from(to);
        local(move || std::fs::rename(&from, &to)).await
    }
}

/// Runs `operation` on a thread where blocking is allowed.
async fn local<T: Send + 'static>(
    operation: impl FnOnce() -> std::io::Result<T> + Send + 'static,
) -> Result<T, String> {
    #[cfg(feature = "real")]
    {
        blocking::unblock(operation)
            .await
            .map_err(|err| err.to_string())
    }
    #[cfg(feature = "mock")]
    {
        let _ = operation;
        Err("Mock mode".to_string())
    }
}

/// Permissions mode, where there is one.
fn mode(permissions: &std::fs::Permissions) -> Option<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Some(permissions.mode())
    }
    #[cfg(not(unix))]
    {
        let _ = permissions;
        None
    }
}

/// Sets the permissions `mode` of `path`, on systems having modes.
fn set_mode(path: &Path, mode: u32) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
        Ok(())
    }
}

/// Create a local filesystem handle optionally rooted at `path`.
///
/// When `path` is `Some`, all file operations performed through the returned `FileSystem`
/// are interpreted relative to that directory, and stay within it: `..`, absolute paths and
/// links leading out of it are refused, links staying within it are followed.
/// When `path` is `None`, paths are those of the process, absolute or relative to its working
/// directory.
///
/// Returns `None` only when the underlying engine cannot be initialised, which in practice
/// never happens for a local filesystem.
#[mel_function]
pub fn local_filesystem(path: Option<string>) -> Option<FileSystem> {
    #[cfg(feature = "real")]
    if let Some(path) = path {
        return Some(FileSystem {
            filesystem: Arc::new(crate::rooted::RootedFileSystemEngine::new(PathBuf::from(
                path,
            ))),
        });
    }
    Some(FileSystem {
        filesystem: Arc::new(LocalFileSystemEngine),
    })
}

#[cfg(all(test, feature = "real", unix))]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::time::{Duration, SystemTime};

    #[test]
    fn entries_are_described_moved_and_removed() {
        let base = std::env::temp_dir().join(format!("fs-mel-local-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("dir")).unwrap();
        std::fs::write(base.join("dir/file"), b"local").unwrap();
        let engine = LocalFileSystemEngine;
        let path = |name: &str| base.join(name).to_string_lossy().to_string();

        block_on(engine.create_link("/nowhere", &path("dir/link"))).unwrap();
        assert_eq!(
            block_on(engine.read_link(&path("dir/link"))).unwrap(),
            "/nowhere"
        );
        assert_eq!(
            block_on(engine.metadata(&path("dir/link"))).unwrap().kind,
            EntryKind::Link
        );
        let mut names = block_on(engine.read_dir(&path("dir"))).unwrap();
        names.sort();
        assert_eq!(names, ["file", "link"]);

        let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        block_on(engine.set_metadata(&path("dir/file"), Some(0o600), Some(modified))).unwrap();
        let file = block_on(engine.metadata(&path("dir/file"))).unwrap();
        assert_eq!((file.kind, file.size), (EntryKind::File, 5));
        assert_eq!(file.mode.unwrap() & 0o777, 0o600);
        assert_eq!(file.modified, Some(modified));

        block_on(engine.rename(&path("dir"), &path("moved"))).unwrap();
        assert!(block_on(engine.remove(&path("moved"), false)).is_err());
        block_on(engine.remove(&path("moved"), true)).unwrap();
        assert!(!base.join("moved").exists());
        std::fs::remove_dir_all(&base).unwrap();
    }
}
