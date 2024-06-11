use crate::messages::Message;
use async_std::io::{BufReader, BufWriter, Read, ReadExt, Write, WriteExt};
use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::sync::Mutex;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(async_std::io::Error),
    Deserialization(ciborium::de::Error<std::io::Error>),
    Serialization(ciborium::ser::Error<std::io::Error>),
}

impl From<async_std::io::Error> for Error {
    fn from(value: async_std::io::Error) -> Self {
        Error::Io(value)
    }
}

pub struct Protocol<R: Read + Write + Unpin> {
    reader: Mutex<BufReader<R>>,
    writer: Mutex<BufWriter<R>>,
}

impl<R: Read + Write + Clone + Unpin> Protocol<R> {
    pub fn new(rw: R) -> Self {
        Self {
            reader: Mutex::new(BufReader::new(rw.clone())),
            writer: Mutex::new(BufWriter::new(rw)),
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
                Ok(())
            }
            Err(err) => Err(Error::Serialization(err)),
        }
    }
}
