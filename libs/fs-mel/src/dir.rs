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
            .create_dir(&path, recursive, &success, &failure, &error)
            .await
    }
}

/// Scan dir contents.
///
/// Each entry of the dir is streamed through `entries`.
/// Once dir is totally scanned, `success` is emitted.  
/// The scanning behavior is set up by the parameters:
/// - `recursive`: set wether subdirectories must be scanned or not,
/// - `follow_links`: set if symbolic links must be followed or not.
///
/// When a scan error happen, it is send through `error` stream.
#[mel_treatment(
    default recursive false
    default follow_links true
    input path Block<string>
    input filesystem Block<FileSystem>
    output entries Stream<string>
    output success Block<void>
    output error Stream<string>
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
            .scan_dir(&path, recursive, follow_links, &entries, &success, &error)
            .await
    }
}
