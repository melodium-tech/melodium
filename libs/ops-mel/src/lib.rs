
//!
//! # Mélodium core operations library
//! 
//! This library provides the basic mathematical operations as functions and treatments for the Mélodium environment, notably:
//! - arithmetic functions;
//! - trigonometric functions;
//! - comparison operations;
//! - logical operations.
//! 
//! ## For Mélodium project
//! 
//! This library is made for use within the Mélodium environment and has no purpose for pure Rust projects.
//! Please refer to the [Mélodium Project](https://melodium.tech/) or
//! the [Mélodium crate](https://docs.rs/melodium/latest/melodium/) for more accurate and detailed information..
//! 

use melodium_macro::mel_package;

pub mod bool;
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
pub mod string;

mel_package!();

