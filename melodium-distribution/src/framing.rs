use melodium_share::RawValue;
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
/// chunked traffic never gets anywhere near the hard ceiling above.
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

/// Greedily groups `values` into chunks whose estimated size stays under `max_bytes`,
/// each becoming one `InputData`/`OutputData` message instead of a single message
/// carrying the whole batch. Every chunk holds at least one value, even if that one value
/// alone exceeds `max_bytes` — this bounds *batches*, not individual values; one huge
/// value still needs application-level splitting, same as an unchunked single value would.
/// An empty input yields no chunks (nothing to send).
pub fn chunk_raw_values(values: Vec<RawValue>, max_bytes: usize) -> Vec<Vec<RawValue>> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_bytes = 0usize;

    for value in values {
        let value_bytes = value.estimated_size();
        if !current.is_empty() && current_bytes + value_bytes > max_bytes {
            chunks.push(std::mem::take(&mut current));
            current_bytes = 0;
        }
        current_bytes += value_bytes;
        current.push(value);
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod chunk_raw_values_tests {
    use super::*;

    fn bytes_of(n: usize) -> RawValue {
        RawValue::Data(
            melodium_share::Identifier {
                version: None,
                path: vec!["root".to_string()],
                name: "Blob".to_string(),
            },
            Some(vec![0u8; n]),
        )
    }

    #[test]
    fn empty_input_yields_no_chunks() {
        assert!(chunk_raw_values(Vec::new(), 1024).is_empty());
    }

    #[test]
    fn small_batch_stays_in_one_chunk() {
        let values = vec![RawValue::U64(1), RawValue::U64(2), RawValue::U64(3)];
        let chunks = chunk_raw_values(values, 1024 * 1024);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 3);
    }

    #[test]
    fn large_batch_splits_into_multiple_bounded_chunks() {
        // Each element is ~100 bytes of payload; capping chunks at 250 bytes should force
        // roughly one element per chunk given the enum's own inline footprint too.
        let values: Vec<_> = (0..5).map(|_| bytes_of(100)).collect();
        let chunks = chunk_raw_values(values, 250);

        assert!(chunks.len() > 1, "expected the batch to actually split");
        let total_values: usize = chunks.iter().map(Vec::len).sum();
        assert_eq!(total_values, 5, "no value should be dropped by chunking");
        for chunk in &chunks {
            assert!(!chunk.is_empty());
        }
    }

    #[test]
    fn a_single_oversized_value_still_gets_through_alone() {
        // One value bigger than max_bytes must still form its own chunk rather than being
        // dropped or causing an infinite loop — batches are bounded, not individual values.
        let values = vec![bytes_of(10_000)];
        let chunks = chunk_raw_values(values, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 1);
    }

    #[test]
    fn an_oversized_value_does_not_get_merged_with_neighbors() {
        let values = vec![RawValue::U64(1), bytes_of(10_000), RawValue::U64(2)];
        let chunks = chunk_raw_values(values, 100);

        // The oversized value must end up alone in its own chunk, not padded out with
        // neighbors that would push that chunk further over the limit.
        let oversized_chunk = chunks
            .iter()
            .find(|c| c.iter().any(|v| matches!(v, RawValue::Data(_, _))))
            .expect("oversized value should be present in some chunk");
        assert_eq!(oversized_chunk.len(), 1);
    }
}
