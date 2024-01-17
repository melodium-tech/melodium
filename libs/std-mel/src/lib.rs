#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_macro::mel_package;

pub mod conv;
pub mod engine;
pub mod flow;
pub mod ops;
pub mod text;
pub mod types;

mel_package!();