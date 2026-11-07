//! Compression with zstd, of data streams while they flow.
//!
//! Data is compressed and decompressed as it comes, keeping in memory only what zstd needs
//! for its window, whatever the size of the stream.

use crate::coding::{code, Coder, Outcome};
use melodium_core::*;
use melodium_macro::mel_treatment;

/// Lowest compression level taken.
const MIN_LEVEL: i32 = 1;
/// Highest compression level taken.
const MAX_LEVEL: i32 = 22;

/// Compresses data with zstd.
///
/// The bytes received through `data` are compressed with zstd at `level`, from 1 (fastest) to 22 (smallest), 3 by default, and given through `compressed` while they come, as one zstd frame.
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
        compressor(level)
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
/// Once `data` ends with zstd frames and everything is given, `completed` is emitted.
/// If `data` is not valid zstd, or ends before its end, `failed` is emitted and `error`
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
        decompressor(),
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

#[cfg(feature = "real")]
pub(crate) fn compressor(level: i32) -> Result<Coder, String> {
    Ok(Coder::new("zstd", |sink| {
        // With a checksum of the content, a changed frame fails to decompress instead of
        // giving other bytes.
        async_compression::futures::write::MzstdEncoder::with_quality_and_params(
            sink,
            async_compression::Level::Precise(level),
            &[async_compression::mzstd::CParameter::checksum_flag(true)],
        )
    }))
}

#[cfg(feature = "mock")]
pub(crate) fn compressor(_level: i32) -> Result<Coder, String> {
    Coder::unavailable("zstd compression")
}

#[cfg(feature = "real")]
pub(crate) fn decompressor() -> Result<Coder, String> {
    Ok(Coder::new("zstd", async_compression::futures::write::MzstdDecoder::new))
}

#[cfg(feature = "mock")]
pub(crate) fn decompressor() -> Result<Coder, String> {
    Coder::unavailable("zstd decompression")
}
