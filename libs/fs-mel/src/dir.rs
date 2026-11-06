use crate::filesystem::*;
use melodium_core::*;
use melodium_macro::mel_treatment;
use std::sync::Arc;

/// Create directory.
///
/// Create the directory specified in `path`, with creation rules depending on parameter:
/// - `recursive`: when `true`, the whole path is targeting the directory is created, and no errors are emitted if
/// the directory already exists; when `false`, only the final directory in the path is created, and errors are emitted
/// if a parent is missing, or the final directory already exists.
///
/// If creation error happens, `failure` is emitted and `error` contains text of the related text of error(s).
#[mel_treatment(
    default recursive true
    input path Block<string>
    input filesystem Block<FileSystem>
    output success Block<void>
    output failure Block<void>
    output error Block<string>
)]
pub async fn create(recursive: bool) {
    if let (Ok(filesystem), Ok(path)) = (
        filesystem.recv_one_as::<Arc<FileSystem>>().await,
        path.recv_one_as::<string>().await,
    ) {
        filesystem
            .filesystem
            .create_dir(
                &path,
                recursive,
                Box::new(|| {
                    Box::pin(async {
                        let _ = success.send_one_as(()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = failure.send_one_as(()).await;
                    })
                }),
                Box::new(|msg: String| {
                    Box::pin(async {
                        let _ = error.send_one_as(msg).await;
                    })
                }),
            )
            .await
    }
}

/// Scan dir contents.
///
/// Each entry of the dir is streamed through `entries`.
/// When directory is totally scanned `finished` is emitted,
/// if without failure `completed` is emitted,
/// and if failure occurs `failed` is emitted.
///
/// The scanning behavior is set up by the parameters:
/// - `recursive`: set wether subdirectories must be scanned or not,
/// - `follow_links`: set if symbolic links must be followed or not.
///
/// When scan errors happen, these are sent through `error` stream.
#[mel_treatment(
    default recursive false
    default follow_links true
    input path Block<string>
    input filesystem Block<FileSystem>
    output entries Stream<string>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output errors Stream<string>
)]
pub async fn scan(recursive: bool, follow_links: bool) {
    if let (Ok(filesystem), Ok(path)) = (
        filesystem.recv_one_as::<Arc<FileSystem>>().await,
        path.recv_one_as::<string>().await,
    ) {
        filesystem
            .filesystem
            .scan_dir(
                &path,
                recursive,
                follow_links,
                Box::new(|path: String| {
                    Box::pin(async { entries.send_one_as(path).await.map_err(|_| ()) })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = completed.send_one_as(()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = failed.send_one_as(()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = finished.send_one_as(()).await;
                    })
                }),
                Box::new(|msg: String| {
                    Box::pin(async { errors.send_one_as(msg).await.map_err(|_| ()) })
                }),
            )
            .await
    }
}

/// Remove a directory.
///
/// The directory at `path` is removed, with all its content when `recursive` (the default),
/// or only when empty otherwise. Links within it are removed themselves, never what they point
/// to.
///
/// `completed` is emitted when the directory is removed. If it cannot be, `failed` is emitted
/// and `error` contains the reason. `finished` is emitted in both cases.
#[mel_treatment(
    default recursive true
    input path Block<string>
    input filesystem Block<FileSystem>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
)]
pub async fn remove(recursive: bool) {
    if let (Ok(filesystem), Ok(path)) = (
        filesystem.recv_one_as::<Arc<FileSystem>>().await,
        path.recv_one_as::<string>().await,
    ) {
        let removed = match filesystem.filesystem.metadata(&path).await {
            Ok(metadata) if metadata.kind != EntryKind::Directory => {
                Err(format!("'{path}' is not a directory"))
            }
            Ok(_) => filesystem.filesystem.remove(&path, recursive).await,
            Err(err) => Err(err),
        };
        match removed {
            Ok(()) => {
                let _ = completed.send_one_as(()).await;
            }
            Err(err) => {
                let _ = failed.send_one_as(()).await;
                let _ = error.send_one_as(err).await;
            }
        }
    }
    let _ = finished.send_one_as(()).await;
}

/// List the files of a directory.
///
/// The files under the directory `path` of `filesystem`, at any depth, are given through
/// `files`, as paths relative to `path` separated by `/`, sorted by name, a directory's files
/// coming where its name sorts. Only regular files are given: directories are walked, while
/// links, never followed, and other entries (devices, sockets, pipes) are left out.
///
/// Once every file is given, `completed` is emitted. If `path` is not a directory or an entry
/// cannot be read, `failed` is emitted and `error` contains the reason. `finished` is emitted in
/// every case.
///
/// ```mermaid
/// graph LR
///     T("files()")
///     F["〈🟨〉"] -->|filesystem| T
///     P["〈🟨〉"] -->|path| T
///     T -->|files| O["… a.txt sub/b.txt"]
///     T -->|completed| C["〈🟩〉"]
///     style F fill:#ffffff,stroke:#ffffff
///     style P fill:#ffffff,stroke:#ffffff
///     style O fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input filesystem Block<FileSystem>
    input path Block<string>
    output files Stream<string>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
)]
pub async fn files() {
    if let (Ok(filesystem), Ok(path)) = (
        filesystem.recv_one_as::<Arc<FileSystem>>().await,
        path.recv_one_as::<string>().await,
    ) {
        let result = walk_files(&*filesystem.filesystem, &path, |file| async {
            files.send_one_as(file).await.is_ok()
        })
        .await;
        files.close().await;
        match result {
            Ok(true) => {
                let _ = completed.send_one_as(()).await;
            }
            Ok(false) => {}
            Err(err) => {
                let _ = failed.send_one_as(()).await;
                let _ = error.send_one_as(err).await;
            }
        }
    }
    let _ = finished.send_one_as(()).await;
}

/// Gives the regular files under `root` to `give`, relative and sorted, as `files` tells,
/// stopping when `give` refuses them.
pub(crate) async fn walk_files<F, Fut>(
    filesystem: &dyn FileSystemEngine,
    root: &str,
    mut give: F,
) -> Result<bool, String>
where
    F: FnMut(String) -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    if filesystem.metadata(root).await?.kind != EntryKind::Directory {
        return Err(format!("'{root}' is not a directory"));
    }
    let full = |relative: &str| {
        if relative.is_empty() {
            root.to_string()
        } else {
            format!("{}/{relative}", root.trim_end_matches('/'))
        }
    };
    let children = |relative: String, mut names: Vec<String>| {
        names.sort_unstable_by(|a, b| b.cmp(a));
        names
            .into_iter()
            .map(move |name| {
                if relative.is_empty() {
                    name
                } else {
                    format!("{relative}/{name}")
                }
            })
            .collect::<Vec<_>>()
    };

    // Entries left to walk, the next one last.
    let mut pending = children(String::new(), filesystem.read_dir(root).await?);
    while let Some(relative) = pending.pop() {
        let path = full(&relative);
        match filesystem.metadata(&path).await?.kind {
            EntryKind::Directory => {
                let names = filesystem.read_dir(&path).await?;
                pending.extend(children(relative, names));
            }
            EntryKind::File => {
                if !give(relative).await {
                    return Ok(false);
                }
            }
            EntryKind::Link | EntryKind::Other => {}
        }
    }
    Ok(true)
}

#[cfg(all(test, feature = "real", unix))]
mod tests {
    use super::*;
    use crate::rooted::RootedFileSystemEngine;
    use futures::executor::block_on;

    #[test]
    fn regular_files_are_listed_sorted() {
        let base = std::env::temp_dir().join(format!("fs-mel-dir-files-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("root");
        std::fs::create_dir_all(root.join("b/deep")).unwrap();
        std::fs::create_dir_all(root.join("empty")).unwrap();
        std::fs::write(root.join("c.txt"), b"c").unwrap();
        std::fs::write(root.join("a.txt"), b"a").unwrap();
        std::fs::write(root.join("b/deep/d.txt"), b"d").unwrap();
        std::fs::write(root.join("b/a.txt"), b"ba").unwrap();
        std::os::unix::fs::symlink("a.txt", root.join("link")).unwrap();
        std::os::unix::fs::symlink("b", root.join("linked_dir")).unwrap();
        let engine = RootedFileSystemEngine::new(base.clone());

        let mut files = Vec::new();
        let result = block_on(walk_files(&engine, "root", |file| {
            files.push(file);
            async { true }
        }));
        assert_eq!(result, Ok(true));
        assert_eq!(files, ["a.txt", "b/a.txt", "b/deep/d.txt", "c.txt"]);

        assert!(block_on(walk_files(&engine, "root/a.txt", |_| async { true })).is_err());
        assert!(block_on(walk_files(&engine, "../outside", |_| async { true })).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }
}
