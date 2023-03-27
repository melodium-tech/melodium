
# Mélodium engine crate

Mélodium core engine implementation.

This crate provides the core Mélodium engine.
The `descriptor` module provides descriptors allowing design to be made.
Everything needed to design `model`s and `treatment`s is provided in the `designer` module.
The `design` module provides purely descriptive design without mutable interaction.

The `engine` trait provides interactions with a core Mélodium engine, that can be instancied through `new_engine` function.

Look at the [Mélodium crate](https://docs.rs/melodium/latest/melodium/)
or the [Mélodium Project](https://melodium.tech/) for more detailed information.
