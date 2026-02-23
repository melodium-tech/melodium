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
pub mod reporting;
pub mod resources;

#[cfg(feature = "kubernetes")]
mod kube;

use melodium_macro::mel_package;

pub(crate) const USER_AGENT: &str = concat!("work-mel/", env!("CARGO_PKG_VERSION"));
pub(crate) static API_URL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("MELODIUM_API_URL")
        .unwrap_or_else(|_| "https://api.melodium.tech/0.1".to_string())
});
pub(crate) static API_TOKEN: std::sync::LazyLock<Option<String>> =
    std::sync::LazyLock::new(|| std::env::var("MELODIUM_API_TOKEN").ok());

mel_package!();
