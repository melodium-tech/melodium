//! Mélodium main library and binary
//!
//! Please refer to the [Mélodium Project](https://melodium.tech/),
//! [Mélodium Book](https://doc.melodium.tech/book/),
//! or [Mélodium Documentation](https://doc.melodium.tech/latest/) for usage.
//!
//! Please refer to the [crates.io page](https://crates.io/crates/melodium) or
//! [project repository](https://gitlab.com/melodium/melodium) for compilation or development information.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_common::{
    descriptor::{Collection, Identifier, LoadingError, LoadingResult, Package},
    executive::Value,
};
use melodium_engine::LogicResult;
pub use melodium_loader::LoadingConfig;
use melodium_loader::{Loader, PackageInfo};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const TARGET: &'static str = env!("TARGET");
pub const TARGET_FEATURES: &'static str = env!("TARGET_FEATURE");
pub const BUILD_HOST: &'static str = env!("HOST");

pub fn load_all(mut config: LoadingConfig) -> LoadingResult<Arc<Collection>> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader.load_all().and_then(|_| loader.build())
}

/*

This former function is kept as implementation idea for direct load and launch of directly available elements.

pub fn load_entry(
    mut config: LoadingConfig,
    identifier: &Identifier,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());


    let loader = Loader::new(config);
    loader
        .load_package(&PackageRequirement {
            package: identifier.root().to_string(),
            version_requirement: VersionReq::parse(">=0.0.0").unwrap(),
        })
        .and_then(|_| loader.load(identifier))
        .and_then(|_| loader.build())
}*/

pub fn load_raw(
    raw: Arc<Vec<u8>>,
    entrypoint: &str,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader
        .load_raw(raw)
        .and_then(|pkg| {
            if let Some(main) = pkg.entrypoints().get(entrypoint) {
                loader.load(&main).and(LoadingResult::new_success(pkg))
            } else {
                LoadingResult::new_failure(LoadingError::no_entry_point_provided(238))
            }
        })
        .and_then(|pkg| {
            loader
                .build()
                .and_then(|collection| LoadingResult::new_success((pkg, collection)))
        })
}

pub fn load_raw_force_entrypoint(
    raw: Arc<Vec<u8>>,
    identifier: &Identifier,
    mut config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader
        .load_raw(raw)
        .and_then(|pkg| {
            loader
                .load(&identifier)
                .and(LoadingResult::new_success(pkg))
        })
        .and_then(|pkg| {
            loader
                .build()
                .and_then(|collection| LoadingResult::new_success((pkg, collection)))
        })
}

pub fn load_file(
    file: PathBuf,
    entrypoint: &str,
    config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    match std::fs::read(&file) {
        Ok(content) => load_raw(Arc::new(content), entrypoint, config),
        Err(err) => {
            LoadingResult::new_failure(LoadingError::unreachable_file(193, file, err.to_string()))
        }
    }
}

pub fn load_file_force_entrypoint(
    file: PathBuf,
    identifier: &Identifier,
    config: LoadingConfig,
) -> LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)> {
    match std::fs::read(&file) {
        Ok(content) => load_raw_force_entrypoint(Arc::new(content), identifier, config),
        Err(err) => {
            LoadingResult::new_failure(LoadingError::unreachable_file(243, file, err.to_string()))
        }
    }
}

pub fn launch(
    collection: Arc<Collection>,
    identifier: &Identifier,
    parameters: HashMap<String, Value>,
) -> LogicResult<()> {
    let engine = melodium_engine::new_engine(collection);
    engine.genesis(&identifier, parameters).and_then(|_| {
        engine.live();
        engine.end();
        LogicResult::new_success(())
    })
}

pub fn core_config() -> LoadingConfig {
    LoadingConfig {
        core_packages: core_packages(),
        search_locations: Vec::new(),
        raw_elements: Vec::new(),
    }
}

pub fn core_packages() -> Vec<Arc<dyn Package>> {
    #[allow(unused_mut)]
    let mut packages = Vec::new();
    packages.push(std_mel::__mel_package::package());

    #[cfg(feature = "encoding-mel")]
    packages.push(encoding_mel::__mel_package::package());
    #[cfg(feature = "fs-mel")]
    packages.push(fs_mel::__mel_package::package());
    #[cfg(feature = "http-mel")]
    packages.push(http_mel::__mel_package::package());
    #[cfg(feature = "javascript-mel")]
    packages.push(javascript_mel::__mel_package::package());
    #[cfg(feature = "json-mel")]
    packages.push(json_mel::__mel_package::package());
    #[cfg(feature = "regex-mel")]
    packages.push(regex_mel::__mel_package::package());

    packages
}
