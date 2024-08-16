use crate::messages::Message;
use async_std::io::{BufReader, BufWriter, Read, Write, WriteExt};
use async_std::sync::Mutex;
use futures::io::{ReadHalf, WriteHalf, AsyncReadExt};
use core::fmt::Display;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(async_std::io::Error),
    Deserialization(ciborium::de::Error<std::io::Error>),
    Serialization(ciborium::ser::Error<std::io::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{err}"),
            Error::Deserialization(err) => write!(f, "{err}"),
            Error::Serialization(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Deserialization(err) => Some(err),
            Error::Serialization(err) => Some(err),
        }
    }
}

impl From<async_std::io::Error> for Error {
    fn from(value: async_std::io::Error) -> Self {
        Error::Io(value)
    }
}

#[derive(Debug)]
pub struct Protocol<R: Read + Write + Unpin + Send> {
    reader: Mutex<BufReader<ReadHalf<R>>>,
    writer: Mutex<BufWriter<WriteHalf<R>>>,
}

impl<R: Read + Write + Unpin + Send> Protocol<R> {
    pub fn new(rw: R) -> Self {
        let (read, write) = rw.split();
        Self {
            reader: Mutex::new(BufReader::new(read)),
            writer: Mutex::new(BufWriter::new(write)),
        }
    }

    pub async fn recv_message(&self) -> Result<Message> {
        let mut reader = self.reader.lock().await;
        let mut expected_size: [u8; 4] = [0; 4];
        reader.read_exact(&mut expected_size).await?;
        let expected_size = u32::from_be_bytes(expected_size) as usize;

        let mut data = vec![0u8; expected_size];
        reader.read_exact(&mut data).await?;

        match ciborium::de::from_reader(data.as_slice()) {
            Ok(message) => Ok(message),
            Err(err) => Err(Error::Deserialization(err)),
        }
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        let mut writer = self.writer.lock().await;

        let mut data = Vec::new();
        match ciborium::into_writer(&message, &mut data) {
            Ok(()) => {
                writer.write_all(&(data.len() as u32).to_be_bytes()).await?;
                writer.write_all(&data).await?;
                writer.flush().await?;
                Ok(())
            }
            Err(err) => Err(Error::Serialization(err)),
        }
    }
}
