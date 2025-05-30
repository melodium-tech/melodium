#![allow(non_camel_case_types)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub use erased_serde::{
    deserialize as erased_deserialize, Error as ErasedSerdeError, Serialize as ErasedSerialize,
    Serializer as ErasedSerializer,
};
pub use melodium_common as common;
pub use once_cell::sync::Lazy;
pub use serde::{Deserialize, Serialize};
pub mod descriptor;
pub mod executive;

pub use melodium_common::executive::{Data, DataTrait, GetData, TransmissionValue, Value};
pub use std::collections::VecDeque;
pub type u8 = core::primitive::u8;
pub type u16 = core::primitive::u16;
pub type u32 = core::primitive::u32;
pub type u64 = core::primitive::u64;
pub type u128 = core::primitive::u128;
pub type i8 = core::primitive::i8;
pub type i16 = core::primitive::i16;
pub type i32 = core::primitive::i32;
pub type i64 = core::primitive::i64;
pub type i128 = core::primitive::i128;
pub type f32 = core::primitive::f32;
pub type f64 = core::primitive::f64;
pub type char = core::primitive::char;
pub type string = std::string::String;
pub type byte = core::primitive::u8;
pub type bool = core::primitive::bool;
pub type void = ();
