//! Compression with deflate64, of data streams while they flow.
//!
//! Data is decompressed as it comes, keeping in memory only what deflate64 needs
//! for its window, whatever the size of the stream.

use crate::coding::{code, Coder, Outcome};
use melodium_macro::mel_treatment;

/// Decompresses data compressed with deflate64.
///
/// The bytes received through `data`, compressed with deflate64, are decompressed and given through
/// `decompressed` while they come.
///
/// Once `data` ends with a complete deflate64 stream and everything is given, `completed` is emitted.
/// If `data` is not valid deflate64, or ends before its end, `failed` is emitted and `error`
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
pub(crate) fn decompressor() -> Result<Coder, String> {
    Ok(Coder::new("deflate64", async_compression::futures::write::Deflate64Decoder::new))
}

#[cfg(feature = "mock")]
pub(crate) fn decompressor() -> Result<Coder, String> {
    Coder::unavailable("deflate64 decompression")
}
