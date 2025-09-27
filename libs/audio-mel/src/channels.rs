use async_channel_io::ChannelReader;
use async_channel::Receiver;
use futures::{AsyncRead, AsyncSeek};
use symphonia::core::io::AsyncMediaSource;

pub struct ChannelReaderMediaSource {
    reader: ChannelReader,
}

impl ChannelReaderMediaSource {
    pub fn new(recv: Receiver<Vec<u8>>) -> Self {
        Self {
            reader: ChannelReader::new(recv)
        }
    }
}

impl AsyncRead for ChannelReaderMediaSource {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::pin!(&mut self.reader).as_mut().poll_read(cx, buf)
    }
}

impl AsyncSeek for ChannelReaderMediaSource {
    fn poll_seek(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _pos: std::io::SeekFrom,
    ) -> std::task::Poll<std::io::Result<u64>> {
        std::task::Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::NotSeekable, "Stream not seekable")))
    }
}

impl AsyncMediaSource for ChannelReaderMediaSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}