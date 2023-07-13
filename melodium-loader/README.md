
# Mélodium loader crate

Mélodium loading engine and utilities.

This crate provides loading logic and processing for the Mélodium environment.

Look at the [Mélodium crate](https://docs.rs/melodium/latest/melodium/)
or the [Mélodium Project](https://melodium.tech/) for more detailed information.

## Features

- `script` (enabled by default): enables ability to load Mélodium scripts (see [melodium-lang](https://docs.rs/melodium-lang/latest/melodium_lang/) crate);
- `jeu` (disabled by default): enables management of `.jeu` packages data format;
- `filesystem` (disabled by default): enables ability to reach filesystem (required to load files from disk);
- `network` (disabled by default): enables ability to use network to retrieve packages (see [melodium-repository](https://docs.rs/melodium-repository/latest/melodium_repository/) crate).
