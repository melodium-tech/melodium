#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_macro::mel_package;

pub mod compare;
pub mod compose;
pub mod convert;
pub mod regex;

mel_package!();
