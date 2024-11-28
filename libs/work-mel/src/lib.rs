#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub mod access;
pub mod api;
pub mod distant;
pub mod resources;

use melodium_macro::mel_package;

pub(crate) const USER_AGENT: &str = concat!("work-mel/", env!("CARGO_PKG_VERSION"));

mel_package!();
