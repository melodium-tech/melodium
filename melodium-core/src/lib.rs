#![allow(non_camel_case_types)]

//!
//! Core Mélodium components for compiled libraries.
//! 
//! This crate is aimed to be used by compiled Mélodium libraries,
//! in conjuction with [`melodium-macro`](https://docs.rs/melodium-macro/latest/melodium_macro/).
//! 
//! Core Mélodium types are exposed here, and base descriptors
//! for built-in elements are provided within the `descriptor` module.
//! 
//! Look at the [Mélodium crate](https://docs.rs/melodium/latest/melodium/)
//! or the [Mélodium Project](https://melodium.tech/) for more detailed information.
//! 

pub use melodium_common as common;
pub use once_cell::sync::Lazy;
pub mod descriptor;

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
