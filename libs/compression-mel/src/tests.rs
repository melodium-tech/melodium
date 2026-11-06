//! Round trip and failure tests, for every format.

use crate::coding::Coder;
use crate::{brotli, deflate, deflate64, gzip, zlib, zstd};

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

/// Format able to compress and decompress.
struct Format {
    name: &'static str,
    compressor: fn(i32) -> Result<Coder, String>,
    decompressor: fn() -> Result<Coder, String>,
    levels: [i32; 3],
    /// Whether a change within the data is always detected.
    detects_changes: bool,
}

fn formats() -> Vec<Format> {
    vec![
        Format {
            name: "zstd",
            compressor: zstd::compressor,
            decompressor: zstd::decompressor,
            levels: [1, 3, 19],
            detects_changes: true,
        },
        Format {
            name: "gzip",
            compressor: gzip::compressor,
            decompressor: gzip::decompressor,
            levels: [1, 6, 9],
            detects_changes: true,
        },
        Format {
            name: "zlib",
            compressor: zlib::compressor,
            decompressor: zlib::decompressor,
            levels: [1, 6, 9],
            detects_changes: true,
        },
        Format {
            name: "deflate",
            compressor: deflate::compressor,
            decompressor: deflate::decompressor,
            levels: [1, 6, 9],
            // No checksum in raw deflate.
            detects_changes: false,
        },
        Format {
            name: "brotli",
            compressor: brotli::compressor,
            decompressor: brotli::decompressor,
            levels: [1, 6, 11],
            // No checksum in brotli.
            detects_changes: false,
        },
    ]
}

#[test]
fn data_comes_back_whole() {
    let data = data();
    for format in formats() {
        for level in format.levels {
            let compressed = run(
                (format.compressor)(level).unwrap(),
                &data.chunks(7_919).collect::<Vec<_>>(),
            )
            .unwrap();
            assert!(
                compressed.len() < data.len() / 4,
                "{} level {level}",
                format.name
            );
            let decompressed = run(
                (format.decompressor)().unwrap(),
                &compressed.chunks(1_000).collect::<Vec<_>>(),
            )
            .unwrap();
            assert_eq!(decompressed, data, "{} level {level}", format.name);
        }
    }
}

#[test]
fn empty_data_comes_back_empty() {
    for format in formats() {
        let compressed = run((format.compressor)(6.min(format.levels[2])).unwrap(), &[]).unwrap();
        assert!(!compressed.is_empty(), "{}", format.name);
        assert!(
            run((format.decompressor)().unwrap(), &[&compressed])
                .unwrap()
                .is_empty(),
            "{}",
            format.name
        );
    }
}

#[test]
fn corrupt_data_fails() {
    for format in formats() {
        let compressed = run((format.compressor)(format.levels[1]).unwrap(), &[&data()]).unwrap();

        // Not the format at all.
        assert!(
            run(
                (format.decompressor)().unwrap(),
                &[b"not compressed data at all, whatever the format"]
            )
            .is_err(),
            "{} foreign",
            format.name
        );

        // Cut within the data.
        let cut = &compressed[..compressed.len() / 2];
        assert!(
            run((format.decompressor)().unwrap(), &[cut]).is_err(),
            "{} cut",
            format.name
        );

        if format.detects_changes {
            let mut changed = compressed.clone();
            let middle = changed.len() / 2;
            changed[middle] ^= 0xff;
            assert!(
                run((format.decompressor)().unwrap(), &[&changed]).is_err(),
                "{} changed",
                format.name
            );
        }
    }
}

#[test]
fn deflate64_decompresses() {
    // Deflate64 has no compressor: a stream of stored blocks is valid in it.
    let payload = b"Melodium";
    let mut stream = vec![0x01, payload.len() as u8, 0, !(payload.len() as u8), 0xff];
    stream.extend_from_slice(payload);
    assert_eq!(
        run(deflate64::decompressor().unwrap(), &[&stream]).unwrap(),
        payload
    );
    assert!(run(deflate64::decompressor().unwrap(), &[&stream[..6]]).is_err());
    assert!(run(deflate64::decompressor().unwrap(), &[b"foreign data of no use"]).is_err());
}
