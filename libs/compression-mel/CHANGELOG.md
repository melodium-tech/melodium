# Changelog

## [Unreleased]

- First version: compressing and decompressing data streams while they flow with zstd, gzip, zlib, deflate and brotli (`compression/<format>::compress`, `compression/<format>::decompress`), and decompressing them with deflate64 (`compression/deflate64::decompress`, no compressor existing for it), built on `async-compression`. Every format is pure Rust and works on WebAssembly (#160).
