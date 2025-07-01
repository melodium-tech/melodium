#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

use melodium_macro::mel_package;

pub mod conv;
pub mod data;
pub mod engine;
pub mod flow;
pub mod ops;
pub mod text;
pub mod types;

mel_package!();
