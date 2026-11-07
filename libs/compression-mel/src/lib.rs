#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

use melodium_macro::mel_package;

mod coding;

pub mod brotli;
pub mod deflate;
pub mod deflate64;
pub mod gzip;
pub mod zlib;
pub mod zstd;

#[cfg(all(test, feature = "real"))]
mod tests;

mel_package!();
