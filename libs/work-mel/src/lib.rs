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

use std::sync::LazyLock;

use melodium_macro::mel_package;

pub(crate) const USER_AGENT: &str = concat!("work-mel/", env!("CARGO_PKG_VERSION"));

pub static EXECUTION_JOB_ID: LazyLock<uuid::Uuid> = LazyLock::new(|| {
    std::env::var("MELODIUM_JOB_ID")
        .ok()
        .map(|id| uuid::Uuid::parse_str(&id).ok())
        .flatten()
        .unwrap_or_else(|| uuid::Uuid::new_v4())
});

pub static EXECUTION_GROUP_ID: LazyLock<uuid::Uuid> = LazyLock::new(|| {
    std::env::var("MELODIUM_GROUP_ID")
        .ok()
        .map(|id| uuid::Uuid::parse_str(&id).ok())
        .flatten()
        .unwrap_or_else(|| uuid::Uuid::new_v4())
});

mel_package!();
