use crate::filesystem::*;
use melodium_core::*;
use melodium_macro::mel_treatment;
use std::sync::Arc;

/// Read one file.
///
/// The content of the file given through `path` is streamed through `data`.
/// When file is reached and opened, `reached` is emitted.
/// Once file is totally and succesfully read, `finished` is emitted.
///
/// If any reading failure happens, `failure` is emitted and `error` contains text of the related text of error(s).
#[mel_treatment(
    input path Block<string>
    input filesystem Block<FileSystem>
    output data Stream<byte>
    output reached Block<void>
    output finished Block<void>
    output failure Block<void>
    output error Stream<string>
)]
pub async fn read() {
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
            .read_file(&path, &data, &reached, &finished, &failure, &error)
            .await
    }
}

/// Write one file.
///
/// The bytes received through `data` are written in the file located at `path`.
/// The writing behavior is set up by the parameters:
/// - `append`: bytes are added to the file instead of replacing the existing file;
/// - `create`: if the file does not exists, it is created;
/// - `new`: the file is required to being new, if a file already exists at that path then the writing fails.
///
/// The amount of written bytes is sent through `amount`. There is no guarantee about its increment, as an undefined number of bytes may be written at once.
///
/// `finished` is emitted when successful writting is finished. `failure` is emitted if an error occurs, and `error` contains the related text of error(s).
#[mel_treatment(
    default append false
    default create true
    default new false
    input path Block<string>
    input filesystem Block<FileSystem>
    input data Stream<byte>
    output finished Block<void>
    output failure Block<void>
    output error Stream<string>
    output amount Stream<u128>
)]
pub async fn write(append: bool, create: bool, new: bool) {
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
            .write_file(
                &path, append, create, new, &data, &amount, &finished, &failure, &error,
            )
            .await
    }
}
