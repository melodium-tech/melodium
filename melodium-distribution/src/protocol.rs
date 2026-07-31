use crate::messages::Message;
use async_std::io::{timeout, BufReader, BufWriter, Read, Write};
use async_std::sync::Mutex;
use core::fmt::Display;
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use futures::io::{AsyncReadExt, ReadHalf, WriteHalf};
use futures::AsyncWriteExt;
use std::sync::OnceLock;

type Result<T> = std::result::Result<T, Error>;

const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// Both sides send a `Message::Probe` every 10s specifically to keep this timeout from
/// firing on an otherwise-idle-but-healthy connection (e.g. a worker silently compiling
/// for a while with nothing new to log). This needs real margin over that 10s interval:
/// under load - several simultaneous distributed connections competing for CPU/IO on
/// either end, e.g. multiple concurrent CI workers each running a heavy local build -
/// the probe task itself can be delayed by scheduling jitter, and too short a timeout
/// causes `recv_message` to time out and tear down a perfectly healthy connection
/// mid-run. Overridable through `MELODIUM_DIST_PROTOCOL_TIMEOUT_SECS`, mainly so tests
/// can exercise a stuck `recv_message` without waiting a full minute.
fn timeout_secs() -> u64 {
    static TIMEOUT: OnceLock<u64> = OnceLock::new();
    *TIMEOUT.get_or_init(|| {
        std::env::var("MELODIUM_DIST_PROTOCOL_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
    })
}

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
            let _ = self.send_message(Message::Ended).await;
            // Currently only writer can be closed (reader rely on timeout)
            let mut writer = self.writer.lock().await;
            let _ = writer.close().await;
            self.closed
                .store(true, core::sync::atomic::Ordering::Relaxed);
        }
    }

    pub async fn recv_message(&self) -> Result<Message> {
        let mut reader = self.reader.lock().await;
        let mut expected_size: [u8; 4] = [0; 4];
        timeout(
            Duration::from_secs(timeout_secs()),
            reader.read_exact(&mut expected_size),
        )
        .await?;
        let expected_size = u32::from_be_bytes(expected_size) as usize;

        let mut data = vec![0u8; expected_size];
        timeout(
            Duration::from_secs(timeout_secs()),
            reader.read_exact(&mut data),
        )
        .await?;

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
        let mut writer = self.writer.lock().await;

        let mut data = Vec::new();
        match ciborium::into_writer(&message, &mut data) {
            Ok(()) => {
                timeout(
                    Duration::from_secs(timeout_secs()),
                    writer.write_all(&(data.len() as u32).to_be_bytes()),
                )
                .await?;
                timeout(Duration::from_secs(timeout_secs()), writer.write_all(&data)).await?;
                timeout(Duration::from_secs(timeout_secs()), writer.flush()).await?;
                Ok(())
            }
            Err(err) => Err(Error::Serialization(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DEFAULT_TIMEOUT_SECS;

    /// Both sides probe every 10s to keep an idle-but-healthy connection from
    /// tripping the read timeout. Regression guard: the default timeout must stay
    /// comfortably above that interval, so ordinary scheduling/network jitter under
    /// load doesn't make a healthy connection look dead (see `timeout_secs`'s
    /// doc comment for the full story - this bit us once already, going from 60s
    /// down to 20s).
    #[test]
    fn default_timeout_has_real_margin_over_probe_interval() {
        const PROBE_INTERVAL_SECS: u64 = 10;
        assert!(
            DEFAULT_TIMEOUT_SECS >= PROBE_INTERVAL_SECS * 3,
            "DEFAULT_TIMEOUT_SECS ({DEFAULT_TIMEOUT_SECS}) must stay at least 3x the probe \
             interval ({PROBE_INTERVAL_SECS}s) to tolerate real scheduling jitter"
        );
    }
}
