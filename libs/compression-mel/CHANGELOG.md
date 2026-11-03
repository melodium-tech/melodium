# Changelog

## [Unreleased]

- First version: compressing and decompressing data streams with zstd while they flow (`compression/zstd::compress`, `compression/zstd::decompress`), built on `async-compression`; not available on WebAssembly, zstd being C code (#160).
