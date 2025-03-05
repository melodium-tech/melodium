use crate::messages::Message;
use async_std::io::{BufReader, BufWriter, Read, Write};
use async_std::sync::Mutex;
use core::fmt::Display;
use core::sync::atomic::AtomicBool;
use futures::io::{AsyncReadExt, ReadHalf, WriteHalf};
use futures::AsyncWriteExt;

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
    closed: AtomicBool,
    reader: Mutex<BufReader<ReadHalf<R>>>,
    writer: Mutex<BufWriter<WriteHalf<R>>>,
}

impl<R: Read + Write + Unpin + Send> Protocol<R> {
    pub fn new(rw: R) -> Self {
        let (read, write) = rw.split();
        Self {
            closed: AtomicBool::new(false),
            reader: Mutex::new(BufReader::new(read)),
            writer: Mutex::new(BufWriter::new(write)),
        }
    }

    pub async fn close(&self) {
        if !self.closed.load(core::sync::atomic::Ordering::Relaxed) {
            eprintln!("Closing protocol… {:?}", std::time::SystemTime::now());
            //let mut reader = self.reader.lock().await;
            let _ = self.send_message(Message::Ended).await;
            let mut writer = self.writer.lock().await;
            //reader.close().await;
            let _ = writer.close().await;
            self.closed
                .store(true, core::sync::atomic::Ordering::Relaxed);
            eprintln!("Protocol closed {:?}", std::time::SystemTime::now());
        }
    }

    pub async fn recv_message(&self) -> Result<Message> {
        if self.closed.load(core::sync::atomic::Ordering::Relaxed) {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "closed",
            )));
        }
        let mut reader = self.reader.lock().await;
        let mut expected_size: [u8; 4] = [0; 4];
        eprintln!("Awaiting receiving message {:?}", std::time::SystemTime::now());
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
        if self.closed.load(core::sync::atomic::Ordering::Relaxed) {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "closed",
            )));
        }
        eprintln!("Awaiting lock writer");
        let mut writer = self.writer.lock().await;
        eprintln!("Awaited lock writer {:?}", std::time::SystemTime::now());

        let mut data = Vec::new();
        match ciborium::into_writer(&message, &mut data) {
            Ok(()) => {
                eprintln!("writing_all len");
                writer.write_all(&(data.len() as u32).to_be_bytes()).await?;
                eprintln!("writing_all data");
                writer.write_all(&data).await?;
                eprintln!("flushing");
                writer.flush().await?;
                eprintln!("flushed");
                Ok(())
            }
            Err(err) => Err(Error::Serialization(err)),
        }
    }
}
