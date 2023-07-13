#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_macro::mel_package;

pub mod file;

mel_package!();
