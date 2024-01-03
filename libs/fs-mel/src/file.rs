use async_std::fs::OpenOptions;
use async_std::io::{ReadExt, WriteExt};
use melodium_core::*;
use melodium_macro::{check, mel_treatment};

/// Read one file.
///
/// The content of the file given through `path` is streamed through `data`.
/// Once file is totally read, `success` is emitted.
///
/// If any reading failure happens, `failure` is emitted and `message` contains text of the related text of error(s).
#[mel_treatment(
    input path Block<string>
    output data Stream<byte>
    output success Block<void>
    output failure Block<void>
    output message Stream<string>
)]
pub async fn read() {
    if let Ok(path) = path
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        let file = OpenOptions::new().read(true).open(path).await;
        match file {
            Ok(mut file) => {
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
                            let _ = message.send_one(err.to_string().into()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    let _ = success.send_one(().into()).await;
                }
            }
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = message.send_one(err.to_string().into()).await;
            }
        }
    } else {
        let _ = failure.send_one(().into()).await;
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
/// `success` is emitted when successful writting is finished. `failure` is emitted if an error occurs, and `message` contains the related text of error(s).
#[mel_treatment(
    default append false
    default create true
    default new false
    input path Block<string>
    input data Stream<byte>
    output success Block<void>
    output failure Block<void>
    output message Stream<string>
    output amount Stream<u128>
)]
pub async fn write(append: bool, create: bool, new: bool) {
    if let Ok(path) = path
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
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
                            let _ = message.send_one(err.to_string().into()).await;
                            fail = true;
                            break;
                        }
                    }
                }
                if !fail {
                    let _ = success.send_one(().into()).await;
                }
            }
            Err(err) => {
                let _ = failure.send_one(().into()).await;
                let _ = message.send_one(err.to_string().into()).await;
            }
        }
    } else {
        let _ = failure.send_one(().into()).await;
    }
}
