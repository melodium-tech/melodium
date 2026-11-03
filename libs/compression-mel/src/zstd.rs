//! Compression with zstd, of data streams while they flow.
//!
//! Data is compressed and decompressed as it comes, keeping in memory only what zstd needs
//! for its window, whatever the size of the stream.

use melodium_core::common::executive::{Input, Output};
use melodium_core::*;
use melodium_macro::mel_treatment;

/// Lowest compression level taken.
const MIN_LEVEL: i32 = 1;
/// Highest compression level taken.
const MAX_LEVEL: i32 = 22;

/// Compresses data with zstd.
///
/// The bytes received through `data` are compressed with zstd at `level`, from 1 (fastest) to
/// 22 (smallest), 3 by default, and given through `compressed` while they come, as one zstd
/// frame.
///
/// Once `data` ends and everything is given, `completed` is emitted. If `level` is not
/// supported or compression fails, `failed` is emitted and `error` contains the reason.
/// `finished` is emitted in every case.
///
/// ```mermaid
/// graph LR
///     T("compress()")
///     B["… 🟦 🟦 🟦"] -->|data| T
///
///     T -->|compressed| O["… 🟪 🟪"]
///     T -->|completed| C["〈🟩〉"]
///
///     style B fill:#ffffff,stroke:#ffffff
///     style O fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    default level 3
    input data Stream<byte>
    output compressed Stream<byte>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
)]
pub async fn compress(level: i32) {
    let coder = if (MIN_LEVEL..=MAX_LEVEL).contains(&level) {
        Coder::compressor(level)
    } else {
        Err(format!(
            "compression level {level} is not supported, expected {MIN_LEVEL} to {MAX_LEVEL}"
        ))
    };
    code(
        coder,
        &**data,
        Outcome {
            data: &**compressed,
            completed: &**completed,
            failed: &**failed,
            finished: &**finished,
            error: &**error,
        },
    )
    .await
}

/// Decompresses data compressed with zstd.
///
/// The bytes received through `data`, compressed with zstd, are decompressed and given through
/// `decompressed` while they come.
///
/// Once `data` ends with complete zstd frames and everything is given, `completed` is emitted.
/// If `data` is not valid zstd, or ends within a frame, `failed` is emitted and `error`
/// contains the reason: what was given before does not make the whole data.
/// `finished` is emitted in every case.
///
/// ```mermaid
/// graph LR
///     T("decompress()")
///     B["… 🟪 🟪"] -->|data| T
///
///     T -->|decompressed| O["… 🟦 🟦 🟦"]
///     T -->|completed| C["〈🟩〉"]
///
///     style B fill:#ffffff,stroke:#ffffff
///     style O fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input data Stream<byte>
    output decompressed Stream<byte>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
)]
pub async fn decompress() {
    code(
        Coder::decompressor(),
        &**data,
        Outcome {
            data: &**decompressed,
            completed: &**completed,
            failed: &**failed,
            finished: &**finished,
            error: &**error,
        },
    )
    .await
}

/// Outputs of the compression treatments.
struct Outcome<'a> {
    data: &'a dyn Output,
    completed: &'a dyn Output,
    failed: &'a dyn Output,
    finished: &'a dyn Output,
    error: &'a dyn Output,
}

/// Runs `coder` on the bytes of `input`, giving what it makes through `outcome`.
async fn code(coder: Result<Coder, String>, input: &dyn Input, outcome: Outcome<'_>) {
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
enum Coder {
    Compressor(async_compression::futures::write::ZstdEncoder<Vec<u8>>),
    Decompressor(async_compression::futures::write::ZstdDecoder<Vec<u8>>),
}

#[cfg(feature = "real")]
impl Coder {
    fn compressor(level: i32) -> Result<Self, String> {
        // With a checksum of the content, a changed frame fails to decompress instead of
        // giving other bytes.
        Ok(Self::Compressor(
            async_compression::futures::write::ZstdEncoder::with_quality_and_params(
                Vec::new(),
                async_compression::Level::Precise(level),
                &[async_compression::zstd::CParameter::checksum_flag(true)],
            ),
        ))
    }

    fn decompressor() -> Result<Self, String> {
        Ok(Self::Decompressor(
            async_compression::futures::write::ZstdDecoder::new(Vec::new()),
        ))
    }

    /// Gives `bytes` to the coder, giving what it made from them so far.
    async fn push(&mut self, bytes: &[u8]) -> Result<Vec<u8>, String> {
        use futures::io::AsyncWriteExt;
        match self {
            Self::Compressor(encoder) => {
                encoder
                    .write_all(bytes)
                    .await
                    .map_err(|err| format!("data cannot be compressed: {err}"))?;
                Ok(core::mem::take(encoder.get_mut()))
            }
            Self::Decompressor(decoder) => {
                decoder
                    .write_all(bytes)
                    .await
                    .map_err(|err| format!("data is not valid zstd: {err}"))?;
                Ok(core::mem::take(decoder.get_mut()))
            }
        }
    }

    /// Ends the data, giving the last bytes made.
    async fn finish(self) -> Result<Vec<u8>, String> {
        use futures::io::AsyncWriteExt;
        match self {
            Self::Compressor(mut encoder) => {
                encoder
                    .close()
                    .await
                    .map_err(|err| format!("data cannot be compressed: {err}"))?;
                Ok(encoder.into_inner())
            }
            Self::Decompressor(mut decoder) => {
                decoder
                    .close()
                    .await
                    .map_err(|err| format!("data is not valid zstd: {err}"))?;
                Ok(decoder.into_inner())
            }
        }
    }
}

/// Compression, not available in mock mode.
#[cfg(feature = "mock")]
enum Coder {}

#[cfg(feature = "mock")]
impl Coder {
    fn compressor(_level: i32) -> Result<Self, String> {
        Err("zstd compression is not available in mock mode".to_string())
    }

    fn decompressor() -> Result<Self, String> {
        Err("zstd decompression is not available in mock mode".to_string())
    }

    async fn push(&mut self, _bytes: &[u8]) -> Result<Vec<u8>, String> {
        match *self {}
    }

    async fn finish(self) -> Result<Vec<u8>, String> {
        match self {}
    }
}

#[cfg(all(test, feature = "real"))]
mod tests {
    use super::*;

    /// Runs `coder` on `chunks`, giving everything it made.
    fn run(mut coder: Coder, chunks: &[&[u8]]) -> Result<Vec<u8>, String> {
        futures::executor::block_on(async {
            let mut made = Vec::new();
            for chunk in chunks {
                made.extend(coder.push(chunk).await?);
            }
            made.extend(coder.finish().await?);
            Ok(made)
        })
    }

    /// Data compressing well, but not to nothing.
    fn data() -> Vec<u8> {
        (0..200_000u32)
            .flat_map(|i| (i % 251).to_le_bytes())
            .collect()
    }

    #[test]
    fn data_comes_back_whole() {
        let data = data();
        for level in [MIN_LEVEL, 3, 19] {
            let compressed = run(
                Coder::compressor(level).unwrap(),
                &data.chunks(7_919).collect::<Vec<_>>(),
            )
            .unwrap();
            assert!(compressed.len() < data.len() / 4, "level {level}");
            let decompressed = run(
                Coder::decompressor().unwrap(),
                &compressed.chunks(1_000).collect::<Vec<_>>(),
            )
            .unwrap();
            assert_eq!(decompressed, data, "level {level}");
        }
    }

    #[test]
    fn empty_data_comes_back_empty() {
        let compressed = run(Coder::compressor(3).unwrap(), &[]).unwrap();
        assert!(!compressed.is_empty());
        assert!(run(Coder::decompressor().unwrap(), &[&compressed])
            .unwrap()
            .is_empty());
    }

    #[test]
    fn corrupt_data_fails() {
        let compressed = run(Coder::compressor(3).unwrap(), &[&data()]).unwrap();

        // Not zstd at all.
        assert!(run(Coder::decompressor().unwrap(), &[b"not zstd data at all"]).is_err());

        // Cut within the frame.
        let cut = &compressed[..compressed.len() / 2];
        assert!(run(Coder::decompressor().unwrap(), &[cut]).is_err());

        // Changed within the frame, its checksum then failing.
        let mut changed = compressed.clone();
        let middle = changed.len() / 2;
        changed[middle] ^= 0xff;
        assert!(run(Coder::decompressor().unwrap(), &[&changed]).is_err());
    }
}
