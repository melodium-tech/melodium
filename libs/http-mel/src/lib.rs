#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_macro::mel_package;
pub mod client;
pub mod method;
pub mod server;
pub mod status;

mel_package!();
