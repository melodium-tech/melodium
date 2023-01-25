
pub mod core;
#[cfg(feature = "filesystem")]
pub mod filesystem;
pub mod package;

#[cfg(feature = "filesystem")]
pub use filesystem::FsPackage;
pub use self::core::CorePackage;
pub use package::Package;
