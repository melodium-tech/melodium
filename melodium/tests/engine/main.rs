//! Mélodium programs run by the engine within the test, which then checks what the engine
//! gives: logs, debug events, launch errors, descriptors. The programs are in `scripts/`.
//!
//! These tests share one binary, as each test binary links the whole engine and every
//! package. Tests checking programs from outside, through the `melodium` executable, are in
//! their own files.

mod common;
mod crypto;
mod option_defaults;
mod secret_access;
mod secret_derivation;
mod secret_runtime_derivation;
mod secret_types;
mod secret_vault;
mod world_release;
