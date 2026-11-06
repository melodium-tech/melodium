//! What is shared by the formats: running a coder on a stream of bytes.

use melodium_core::common::executive::{Input, Output};
use melodium_core::*;

/// Outputs of the compression treatments.
pub(crate) struct Outcome<'a> {
    pub(crate) data: &'a dyn Output,
    pub(crate) completed: &'a dyn Output,
    pub(crate) failed: &'a dyn Output,
    pub(crate) finished: &'a dyn Output,
    pub(crate) error: &'a dyn Output,
}

/// Runs `coder` on the bytes of `input`, giving what it makes through `outcome`.
pub(crate) async fn code(coder: Result<Coder, String>, input: &dyn Input, outcome: Outcome<'_>) {
    let Outcome {
        data,
        completed,
        failed,
        finished,
        error,
    } = outcome;

    let result: Result<bool, String> = async {
        let mut coder = coder?;
        while let Ok(bytes) = input.recv_many_as::<u8>().await {
            let coded = coder.push(&bytes).await?;
            if !coded.is_empty()
                && data
                    .send_many(TransmissionValue::Byte(coded.into()))
                    .await
                    .is_err()
            {
                // Nothing takes what is given anymore.
                return Ok(false);
            }
        }
        let coded = coder.finish().await?;
        Ok(coded.is_empty()
            || data
                .send_many(TransmissionValue::Byte(coded.into()))
                .await
                .is_ok())
    }
    .await;

    input.close();
    data.close().await;
    match result {
        Ok(true) => {
            let _ = completed.send_one(().into()).await;
        }
        Ok(false) => {}
        Err(message) => {
            let _ = failed.send_one(().into()).await;
            let _ = error.send_one(message.into()).await;
        }
    }
    let _ = finished.send_one(().into()).await;
}

/// Compression or decompression, made while bytes are given to it.
#[cfg(feature = "real")]
pub(crate) struct Coder {
    name: &'static str,
    writer: core::pin::Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    sink: Sink,
}

/// Where a coder writes what it makes, so that it can be taken while the data flows.
#[cfg(feature = "real")]
#[derive(Clone, Default)]
pub(crate) struct Sink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

#[cfg(feature = "real")]
impl Sink {
    fn take(&self) -> Vec<u8> {
        core::mem::take(&mut *self.0.lock().unwrap())
    }
}

#[cfg(feature = "real")]
impl futures::io::AsyncWrite for Sink {
    fn poll_write(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
        buf: &[u8],
    ) -> core::task::Poll<std::io::Result<usize>> {
        self.0.lock().unwrap().extend_from_slice(buf);
        core::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<std::io::Result<()>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<std::io::Result<()>> {
        core::task::Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "real")]
impl Coder {
    /// Coder of the format `name`, made by `build` from the place it has to write to.
    pub(crate) fn new<W>(name: &'static str, build: impl FnOnce(Sink) -> W) -> Self
    where
        W: futures::io::AsyncWrite + Send + 'static,
    {
        let sink = Sink::default();
        Self {
            name,
            writer: Box::pin(build(sink.clone())),
            sink,
        }
    }

    /// Gives `bytes` to the coder, giving what it made from them so far.
    pub(crate) async fn push(&mut self, bytes: &[u8]) -> Result<Vec<u8>, String> {
        use futures::io::AsyncWriteExt;
        self.writer
            .write_all(bytes)
            .await
            .map_err(|err| format!("data is not valid {}: {err}", self.name))?;
        Ok(self.sink.take())
    }

    /// Ends the data, giving the last bytes made.
    pub(crate) async fn finish(mut self) -> Result<Vec<u8>, String> {
        use futures::io::AsyncWriteExt;
        self.writer
            .close()
            .await
            .map_err(|err| format!("data is not valid {}: {err}", self.name))?;
        Ok(self.sink.take())
    }
}

/// Compression, not available in mock mode.
#[cfg(feature = "mock")]
pub(crate) enum Coder {}

#[cfg(feature = "mock")]
impl Coder {
    pub(crate) fn unavailable(what: &str) -> Result<Self, String> {
        Err(format!("{what} is not available in mock mode"))
    }

    pub(crate) async fn push(&mut self, _bytes: &[u8]) -> Result<Vec<u8>, String> {
        match *self {}
    }

    pub(crate) async fn finish(self) -> Result<Vec<u8>, String> {
        match self {}
    }
}
