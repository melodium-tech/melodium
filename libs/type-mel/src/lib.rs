//!
//! # Mélodium core types utilities library
//! 
//! This library provides the data type utilities for the Mélodium environment.
//! 
//! ## For Mélodium project
//! 
//! This library is made for use within the Mélodium environment and has no purpose for pure Rust projects.
//! Please refer to the [Mélodium Project](https://melodium.tech/) for more accurate and detailed information.
//! 

use melodium_macro::mel_package;
use melodium_core::*;

pub mod byte;
pub mod char;
pub mod u8;
pub mod u16;
pub mod u32;
pub mod u64;
pub mod u128;
pub mod i8;
pub mod i16;
pub mod i32;
pub mod i64;
pub mod i128;
pub mod f32;
pub mod f64;

mel_package!();