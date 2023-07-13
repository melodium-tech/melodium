
# Mélodium repository crate

Mélodium repository utilities.

This crate provides repository logic for the Mélodium environment.

Look at the [Mélodium crate](https://docs.rs/melodium/latest/melodium/)
or the [Mélodium Project](https://melodium.tech/) for more detailed information.

## Features

- `network` (disabled by default): allow network access to retrieve packages;
- `cargo` (disabled by default): allow to extract package informations from `Cargo.toml` files.

## Network security

This crate uses different security implementations depending on platform it is built for.
When built for `apple` targets, it uses the native system TLS implementation.
When build for Windows systems with `msvc` target, it uses the MS TLS implementation.
For all other targets, `rustls` is used.  
Only applicable when `network` feature is enabled.
