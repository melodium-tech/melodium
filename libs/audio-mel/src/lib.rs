#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_package, mel_treatment};
use std::sync::Arc;

pub mod audio_info;
pub(crate) mod channels;
pub mod decode;
pub mod encode;
pub mod transform;

mel_package!();
