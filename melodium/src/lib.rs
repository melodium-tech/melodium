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

use melodium_common::descriptor::{
    Collection, Identifier, LoadingError, LoadingResult, Package, PackageRequirement, VersionReq,
};
use melodium_engine::LogicResult;
use melodium_loader::Loader;
pub use melodium_loader::LoadingConfig;
use std::path::PathBuf;
use std::sync::Arc;

pub fn load_all(mut config: LoadingConfig) -> LoadingResult<Arc<Collection>> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader.load_all().and_then(|_| loader.build())
}

pub fn load_entry(
    mut config: LoadingConfig,
    identifier: &Identifier,
) -> LoadingResult<Arc<Collection>> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader
        .load_package(&PackageRequirement {
            package: identifier.root().to_string(),
            version_requirement: VersionReq::parse(">=0.0.0").unwrap(),
        })
        .and_then(|_| loader.load(identifier))
        .and_then(|_| loader.build())
}

pub fn load_raw(
    raw: Arc<Vec<u8>>,
    main: Option<Identifier>,
    mut config: LoadingConfig,
) -> LoadingResult<(Identifier, Arc<Collection>)> {
    config.extend(core_config());

    let loader = Loader::new(config);
    loader
        .load_raw(raw)
        .and_then(|(_, pkg_main)| {
            if let Some(main) = main.or(pkg_main) {
                loader.load(&main).and(LoadingResult::new_success(main))
            } else {
                LoadingResult::new_failure(LoadingError::no_entry_point_provided(238))
            }
        })
        .and_then(|identifier| {
            loader
                .build()
                .and_then(|collection| LoadingResult::new_success((identifier, collection)))
        })
}

pub fn load_file(
    file: PathBuf,
    main: Option<Identifier>,
    config: LoadingConfig,
) -> LoadingResult<(Identifier, Arc<Collection>)> {
    match std::fs::read(&file) {
        Ok(content) => load_raw(Arc::new(content), main, config),
        Err(err) => {
            LoadingResult::new_failure(LoadingError::unreachable_file(193, file, err.to_string()))
        }
    }
}

pub fn launch(collection: Arc<Collection>, identifier: &Identifier) -> LogicResult<()> {
    let engine = melodium_engine::new_engine(collection);
    engine.genesis(&identifier).and_then(|_| {
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
    #[cfg(feature = "conv-mel")]
    packages.push(conv_mel::__mel_package::package());
    #[cfg(feature = "encoding-mel")]
    packages.push(encoding_mel::__mel_package::package());
    #[cfg(feature = "engine-mel")]
    packages.push(engine_mel::__mel_package::package());
    #[cfg(feature = "flow-mel")]
    packages.push(flow_mel::__mel_package::package());
    #[cfg(feature = "fs-mel")]
    packages.push(fs_mel::__mel_package::package());
    #[cfg(feature = "http-mel")]
    packages.push(http_mel::__mel_package::package());
    #[cfg(feature = "ops-mel")]
    packages.push(ops_mel::__mel_package::package());
    #[cfg(feature = "regex-mel")]
    packages.push(regex_mel::__mel_package::package());
    #[cfg(feature = "text-mel")]
    packages.push(text_mel::__mel_package::package());
    #[cfg(feature = "type-mel")]
    packages.push(type_mel::__mel_package::package());
    packages
}
