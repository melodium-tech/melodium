//! The `compression` package compresses and decompresses data streams.

use super::common;
use melodium_common::executive::Value;
use melodium_engine::debug::Event;
use std::collections::HashMap;

/// Script written for zstd, other formats are made by naming them instead.
const ZSTD: &str = include_str!("scripts/compression_zstd.mel");

/// Bytes sent by the output `output` of the instance labelled `label`.
fn sent_bytes(events: &[Event], label: &str, output: &str) -> Vec<u8> {
    common::sent(events, label, output)
        .into_iter()
        .map(|value| match value {
            Value::Byte(byte) => byte,
            other => panic!("not a byte: {:?}", other),
        })
        .collect()
}

/// Checks the script of `format`, with `level` in range and `bad_level` out of it, giving
/// the bytes the compressed data starts with, if any.
fn check_format(format: &str, level: i32, bad_level: i32, magic: &[u8]) {
    let script = ZSTD
        .replace("zstd", format)
        .replace("level=19", &format!("level={level}"))
        .replace("level=40", &format!("level={bad_level}"));
    let (logs, events) = common::run(&script, HashMap::new());

    let text = "Mélodium streams data through compression and back, Mélodium streams data through compression and back.";
    let compressed = sent_bytes(&events, "compress", "compressed");
    assert_eq!(compressed[..magic.len()], *magic);
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
    assert!(corrupt.contains(&format!("not valid {format}")), "{}", corrupt);

    let unsupported = common::message(&logs, "unsupported");
    assert!(
        unsupported.contains(&format!("compression level {bad_level} is not supported")),
        "{}",
        unsupported
    );
}

#[test]
fn data_is_compressed_with_zstd() {
    // The zstd magic number.
    check_format("zstd", 19, 40, &[0x28, 0xb5, 0x2f, 0xfd]);
}

#[test]
fn data_is_compressed_with_gzip() {
    // The gzip magic number.
    check_format("gzip", 9, 40, &[0x1f, 0x8b]);
}

#[test]
fn data_is_compressed_with_zlib() {
    // The zlib header, with the best compression flag.
    check_format("zlib", 9, 40, &[0x78, 0xda]);
}

#[test]
fn data_is_compressed_with_deflate() {
    check_format("deflate", 9, 40, &[]);
}

#[test]
fn data_is_compressed_with_brotli() {
    check_format("brotli", 11, 40, &[]);
}
