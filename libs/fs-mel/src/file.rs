use crate::filesystem::*;
use melodium_core::*;
use melodium_macro::mel_treatment;
use std::sync::Arc;

/// Read one file.
///
/// The content of the file given through `path` is streamed through `data`.
/// When file is reached and opened, `reached` is emitted.
/// Once file is totally and succesfully read, `completed` is emitted.
/// `finished` is emitted when the read ends, regardless of the reason.
/// All reading errors are streamed through `errors`.
///
/// If any reading failure happens, `failed` is emitted.
#[mel_treatment(
    input path Block<string>
    input filesystem Block<FileSystem>
    output data Stream<byte>
    output reached Block<void>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output errors Stream<string>
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
            .read_file(
                &path,
                Box::new(|content: VecDeque<u8>| {
                    Box::pin(async {
                        data.send_many(TransmissionValue::Byte(content))
                            .await
                            .map_err(|_| ())
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = reached.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        reached.close().await;
                    })
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
/// `completed` is emitted when successful writting is finished. `failed` is emitted if an error occurs, and `errors` contains the related text of error(s).
/// `finished` is emitted at the end, regardless of the writing status.
#[mel_treatment(
    default append false
    default create true
    default new false
    input path Block<string>
    input filesystem Block<FileSystem>
    input data Stream<byte>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output errors Stream<string>
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
                &path,
                append,
                create,
                new,
                Box::new(|| {
                    Box::pin(async {
                        data.recv_many()
                            .await
                            .map(|values| TryInto::<Vec<u8>>::try_into(values).unwrap())
                            .map_err(|_| ())
                    })
                }),
                Box::new(|amt: u128| {
                    Box::pin({
                        let amount = &amount;
                        async move { amount.send_one(amt.into()).await.map_err(|_| ()) }
                    })
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
