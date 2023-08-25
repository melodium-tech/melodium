use async_std::fs;
use async_std::stream::StreamExt;
use async_walkdir::{Filtering, WalkDir};
use melodium_core::*;
use melodium_macro::{check, mel_treatment};

/// Create directory.
///
/// Create the directory specified in `path`, with creation rules depending on parameter:
/// - `recursive`: when `true`, the whole path is targeting the directory is created, and no errors are emitted if
/// the directory already exists; when `false`, only the final directory in the path is created, and errors are emitted
/// if a parent is missing, or the final directory already exists.
///
/// If creation error happens, `failure` is emitted and `message` contains text of the related text of error(s).
#[mel_treatment(
    default recursive true
    input path Block<string>
    output success Block<void>
    output failure Block<void>
    output message Stream<string>
)]
pub async fn create(recursive: bool) {
    if let Ok(path) = path.recv_one_string().await {
        match if recursive {
            fs::create_dir_all(path).await
        } else {
            fs::create_dir(path).await
        } {
            Ok(()) => {
                let _ = success.send_one_void(()).await;
            }
            Err(err) => {
                let _ = message.send_one_string(err.to_string()).await;
                let _ = failure.send_one_void(()).await;
            }
        }
    } else {
        let _ = failure.send_one_void(()).await;
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
/// If any scan error happens, `failure` is emitted and `message` contains text of the related text of error(s).
#[mel_treatment(
    default recursive false
    default follow_links true
    input path Block<string>
    output entries Stream<string>
    output success Block<void>
    output failure Block<void>
    output message Stream<string>
)]
pub async fn scan(recursive: bool, follow_links: bool) {
    if let Ok(path) = path.recv_one_string().await {
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
                        .send_one_string(entry.path().to_string_lossy().to_string())
                        .await
                ),
                Err(err) => {
                    let _ = message.send_one_string(err.to_string()).await;
                    let _ = failure.send_one_void(()).await;
                }
            }
        }
        let _ = success.send_one_void(()).await;
    } else {
        let _ = failure.send_one_void(()).await;
    }
}
