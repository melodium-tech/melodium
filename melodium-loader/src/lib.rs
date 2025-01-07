#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod compo;
mod content;
mod loader;
mod loading_config;
mod package;
mod package_manager;

pub const TRIPLE: &str = env!("TARGET");
pub const LIB_ROOT_FILENAME: &str = "lib-root.mel";

pub use compo::Compo;
pub use loader::Loader;
pub use loading_config::LoadingConfig;
pub use package::PackageInfo;

#[cfg(all(feature = "filesystem", feature = "jeu"))]
pub use package::build_jeu;
#[cfg(all(feature = "filesystem", feature = "jeu"))]
pub use package::extract_jeu;
