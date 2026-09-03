use std::sync::OnceLock;

/// Hard ceiling on a single wire frame, enforced on receipt before any allocation happens.
/// Deliberately generous relative to `max_batch_chunk_bytes` below: this is a safety net
/// against a malicious or misbehaving peer's claimed length (previously trusted up to
/// `u32::MAX`, ~4 GiB), not the normal operating size — legitimate one-off messages that
/// have no chunking mechanism of their own (e.g. `LoadAndLaunch`'s program `Collection`)
/// still need to fit under it whole.
const DEFAULT_MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

/// Soft target used when proactively splitting a data batch into multiple `InputData`/
/// `OutputData` messages at construction time, well under `max_frame_bytes` so normal
/// chunked traffic never gets anywhere near the hard ceiling above. Consumed by
/// `TransmissionValue::chunked` (see `melodium-share`) at each construction site.
const DEFAULT_MAX_BATCH_CHUNK_BYTES: usize = 1024 * 1024;

/// See `DEFAULT_MAX_FRAME_BYTES`. Overridable through
/// `MELODIUM_DIST_PROTOCOL_MAX_FRAME_BYTES`, mainly so tests can exercise the rejection
/// path without allocating tens of megabytes.
pub fn max_frame_bytes() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("MELODIUM_DIST_PROTOCOL_MAX_FRAME_BYTES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_MAX_FRAME_BYTES)
    })
}

/// See `DEFAULT_MAX_BATCH_CHUNK_BYTES`. Overridable through
/// `MELODIUM_DIST_PROTOCOL_MAX_BATCH_CHUNK_BYTES`.
pub fn max_batch_chunk_bytes() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("MELODIUM_DIST_PROTOCOL_MAX_BATCH_CHUNK_BYTES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_MAX_BATCH_CHUNK_BYTES)
    })
}
