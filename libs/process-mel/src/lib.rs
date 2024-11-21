#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub mod command;
pub mod environment;
pub mod exec;
pub mod local;

use melodium_macro::mel_package;

mel_package!();
