//! Mélodium main library and binary
//!
//! Please refer to the [Mélodium Project](https://melodium.tech/),
//! [Mélodium Book](https://doc.melodium.tech/book/),
//! or [Mélodium Documentation](https://doc.melodium.tech/latest/) for usage.
//!
//! Please refer to the [crates.io page](https://crates.io/crates/melodium) or
//! [project repository](https://gitlab.com/melodium/melodium) for compilation or development information.
//!
//!

use melodium_common::descriptor::{Collection, Identifier, LoadingError, Package};
use melodium_loader::{Loader, LoadingConfig};
use std::path::PathBuf;
use std::sync::Arc;

pub fn load() -> Result<Collection, LoadingError> {
    let config = LoadingConfig {
        core_packages: core_packages(),
        search_locations: Vec::new(),
    };

    let loader = Loader::new(config);
    loader.full_load()
}

pub fn load_entry(paths: Vec<PathBuf>, id: &Identifier) -> Result<Arc<Collection>, LoadingError> {
    let config = LoadingConfig {
        core_packages: core_packages(),
        search_locations: paths,
    };

    let loader = Loader::new(config);
    loader.load_package(id.root())?;
    loader.load(id)?;
    loader.build()
}

pub fn load_raw(
    raw: &str,
    paths: Vec<PathBuf>,
) -> Result<(Identifier, Arc<Collection>), LoadingError> {
    let config = LoadingConfig {
        core_packages: core_packages(),
        search_locations: paths,
    };

    let loader = Loader::new(config);
    let name = loader.load_raw(raw)?;
    let identifier = Identifier::new(vec![name], "main");

    loader.load(&identifier)?;

    match loader.build() {
        Ok(collection) => Ok((identifier, collection)),
        Err(err) => Err(err),
    }
}

pub fn load_file(
    file: PathBuf,
    paths: Vec<PathBuf>,
) -> Result<(Identifier, Arc<Collection>), LoadingError> {
    let content = std::fs::read_to_string(file).map_err(|_| LoadingError::ContentError)?;
    load_raw(&content, paths)
}

fn core_packages() -> Vec<Box<dyn Package>> {
    let mut packages = Vec::new();
    #[cfg(feature = "conv-mel")]
    packages.push(conv_mel::__mel_package::package());
    #[cfg(feature = "engine-mel")]
    packages.push(engine_mel::__mel_package::package());
    #[cfg(feature = "flow-mel")]
    packages.push(flow_mel::__mel_package::package());
    #[cfg(feature = "fs-mel")]
    packages.push(fs_mel::__mel_package::package());
    #[cfg(feature = "ops-mel")]
    packages.push(ops_mel::__mel_package::package());
    #[cfg(feature = "type-mel")]
    packages.push(type_mel::__mel_package::package());
    packages
}
