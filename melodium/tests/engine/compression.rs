//! The `compression` package compresses and decompresses data streams with zstd.

use super::common;
use melodium_common::executive::Value;
use melodium_engine::debug::Event;
use std::collections::HashMap;

const ZSTD: &str = include_str!("scripts/compression_zstd.mel");

/// Bytes sent by the output `output` of the instance labelled `label`.
fn sent_bytes(events: &[Event], label: &str, output: &str) -> Vec<u8> {
    common::sent(events, label, output)
        .into_iter()
        .map(|value| match value {
            Value::Byte(byte) => byte,
            other => panic!("not a byte: {other:?}"),
        })
        .collect()
}

#[test]
fn data_is_compressed_with_zstd() {
    let (logs, events) = common::run(ZSTD, HashMap::new());

    let text = "Mélodium streams data through compression and back, Mélodium streams data through compression and back.";
    let compressed = sent_bytes(&events, "compress", "compressed");
    // The zstd magic number.
    assert_eq!(compressed[..4], [0x28, 0xb5, 0x2f, 0xfd]);
    assert!(compressed.len() < text.len(), "{}", compressed.len());
    assert_eq!(
        sent_bytes(&events, "decompress", "decompressed"),
        text.as_bytes()
    );
    assert_eq!(common::sent(&events, "compress", "completed").len(), 1);
    assert_eq!(common::sent(&events, "decompress", "completed").len(), 1);

    assert!(sent_bytes(&events, "corrupt", "decompressed").is_empty());
    assert!(common::sent(&events, "corrupt", "completed").is_empty());
    assert_eq!(common::sent(&events, "corrupt", "failed").len(), 1);
    let corrupt = common::message(&logs, "corrupt");
    assert!(corrupt.contains("not valid zstd"), "{corrupt}");

    let unsupported = common::message(&logs, "unsupported");
    assert!(
        unsupported.contains("compression level 40 is not supported"),
        "{unsupported}"
    );
}
