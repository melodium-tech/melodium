#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "mock", allow(unused))]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

use melodium_macro::mel_package;
pub mod client;
pub mod method;
pub mod server;
pub mod status;

mel_package!();
