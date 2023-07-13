
# Mélodium engine crate

Mélodium core engine implementation.

This crate provides the core Mélodium engine.
The [descriptor](crate::descriptor) module provides descriptors allowing design to be made.
Everything needed to design [models](crate::designer::Model) and [treatments](crate::designer::Treatment) is provided in the [designer](crate::designer) module.
The [design](crate::design) module provides purely descriptive design without mutable interaction.

The [engine](crate::Engine) trait provides interactions with a core Mélodium engine, that can be instancied through [new_engine](crate::new_engine) function.

Look at the [Mélodium crate](https://docs.rs/melodium/latest/melodium/)
or the [Mélodium Project](https://melodium.tech/) for more detailed information.

## Features

- `doc` (disabled by default): enables documentation management of elements, when disabled `documentation` and `set_documentation` functions will still be present but respectively return empty string and do nothing.
