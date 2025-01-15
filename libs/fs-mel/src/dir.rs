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
        filesystem.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<FileSystem>()
                .unwrap()
        }),
        path.recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap()),
    ) {
        filesystem
            .filesystem
            .create_dir(
                &path,
                recursive,
                Box::new(|| {
                    Box::pin(async {
                        let _ = success.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = failure.send_one(().into()).await;
                    })
                }),
                Box::new(|msg: String| {
                    Box::pin(async {
                        let _ = error.send_one(msg.into()).await;
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
        filesystem.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<FileSystem>()
                .unwrap()
        }),
        path.recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap()),
    ) {
        filesystem
            .filesystem
            .scan_dir(
                &path,
                recursive,
                follow_links,
                Box::new(|path: String| {
                    Box::pin(async { entries.send_one(path.into()).await.map_err(|_| ()) })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = completed.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = failed.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = finished.send_one(().into()).await;
                    })
                }),
                Box::new(|msg: String| {
                    Box::pin(async { errors.send_one(msg.into()).await.map_err(|_| ()) })
                }),
            )
            .await
    }
}
