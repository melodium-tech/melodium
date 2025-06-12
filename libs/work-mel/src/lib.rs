#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "mock", allow(unused))]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

pub mod access;
pub mod api;
pub mod compose;
pub mod container;
pub mod distant;
pub mod resources;

#[cfg(feature = "kubernetes")]
mod kube;

use melodium_macro::mel_package;

pub(crate) const USER_AGENT: &str = concat!("work-mel/", env!("CARGO_PKG_VERSION"));

mel_package!();
