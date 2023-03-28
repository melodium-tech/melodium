//!
//! # Mélodium core conversions library
//!
//! This library provides the data type conversion functions and treatments for the Mélodium environment.
//!
//! ## For Mélodium project
//!
//! This library is made for use within the Mélodium environment and has no purpose for pure Rust projects.
//! Please refer to the [Mélodium Project](https://melodium.tech/) or
//! the [Mélodium crate](https://docs.rs/melodium/latest/melodium/) for more accurate and detailed information.
//!

use melodium_macro::mel_package;

pub mod bool;
pub mod byte;
pub mod char;
pub mod f32;
pub mod f64;
pub mod i128;
pub mod i16;
pub mod i32;
pub mod i64;
pub mod i8;
pub mod string;
pub mod u128;
pub mod u16;
pub mod u32;
pub mod u64;
pub mod u8;

mel_package!();
