#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_macro::mel_package;

pub mod dir;
pub mod file;
pub mod filesystem;
pub mod local;
pub mod path;

mel_package!();
