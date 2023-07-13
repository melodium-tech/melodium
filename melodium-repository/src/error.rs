use crate::technical::{Availability, Element, Platform};
use core::fmt::Display;
use melodium_common::descriptor::{RepositoryError as CommonRepositoryError, Version};
use std::path::PathBuf;

#[derive(Debug)]
pub enum RepositoryErrorKind {
    AlreadyExistingPackage {
        package: String,
        version: Version,
    },
    UnknownPackage {
        package: String,
        version: Version,
    },
    FsError {
        error: std::io::Error,
    },
    JsonError {
        error: serde_json::Error,
    },
    NoNetwork,
    NetworkError {
        error: String,
    },
    PlatformDependant {
        package: String,
        version: Version,
    },
    NotPlatformDependant {
        package: String,
        version: Version,
    },
    PlatformUnavailable {
        package: String,
        version: Version,
        platform: Platform,
        availability: Availability,
    },
    PackageElementAbsent {
        package: String,
        version: Version,
        platform: Option<(Platform, Availability)>,
        element: Element,
        path: PathBuf,
    },
}

impl Display for RepositoryErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryErrorKind::AlreadyExistingPackage { package, version } => write!(
                f,
                "Package '{package}' version {version} already present in repository"
            ),
            RepositoryErrorKind::UnknownPackage { package, version } => write!(
                f,
                "Package '{package}' version {version} not found in repository"
            ),
            RepositoryErrorKind::FsError { error } => write!(f, "Filesystem error: {error}"),
            RepositoryErrorKind::JsonError { error } => write!(f, "JSON error: {error}"),
            RepositoryErrorKind::NoNetwork => write!(f, "No network ability for repository call"),
            RepositoryErrorKind::NetworkError { error } => write!(f, "Network error: {error}"),
            RepositoryErrorKind::PlatformDependant { package, version } => write!(f, "Package '{package}' version {version} is platform dependent"),
            RepositoryErrorKind::NotPlatformDependant { package, version } => write!(f, "Package '{package}' version {version} is not platform dependent"),
            RepositoryErrorKind::PlatformUnavailable { package, version, platform, availability } => write!(f, "Package '{package}' version {version} unavailable for '{platform}' as {availability}"),
            RepositoryErrorKind::PackageElementAbsent { package, version, platform, element, path } => match platform {
                Some((platform, availability)) => write!(f, "Element '{element}' of package '{package}' version {version} for '{platform}' ({availability}) is missing locally (looking at '{path}')", element = element.name, path = path.to_string_lossy()),
                None => write!(f, "Element '{element}' of package '{package}' version {version} is missing locally (looking at '{path}')", element = element.name, path = path.to_string_lossy()),
            },
        }
    }
}

#[derive(Debug)]
pub struct RepositoryError {
    pub id: u32,
    pub kind: RepositoryErrorKind,
}
impl RepositoryError {
    pub fn already_existing_package(id: u32, package: String, version: Version) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::AlreadyExistingPackage { package, version },
        }
    }

    pub fn unknown_package(id: u32, package: String, version: Version) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::UnknownPackage { package, version },
        }
    }

    pub fn fs_error(id: u32, error: std::io::Error) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::FsError { error },
        }
    }

    pub fn json_error(id: u32, error: serde_json::Error) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::JsonError { error },
        }
    }

    pub fn no_network(id: u32) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::NoNetwork,
        }
    }

    pub fn network_error(id: u32, error: String) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::NetworkError { error },
        }
    }

    pub fn platform_dependent(id: u32, package: String, version: Version) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::PlatformDependant { package, version },
        }
    }

    pub fn not_platform_dependent(id: u32, package: String, version: Version) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::NotPlatformDependant { package, version },
        }
    }

    pub fn platform_unavailable(
        id: u32,
        package: String,
        version: Version,
        platform: Platform,
        availability: Availability,
    ) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::PlatformUnavailable {
                package,
                version,
                platform,
                availability,
            },
        }
    }

    pub fn package_element_absent(
        id: u32,
        package: String,
        version: Version,
        platform: Option<(Platform, Availability)>,
        element: Element,
        path: PathBuf,
    ) -> Self {
        Self {
            id,
            kind: RepositoryErrorKind::PackageElementAbsent {
                package,
                version,
                platform,
                element,
                path,
            },
        }
    }
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "R{:04}: {}", self.id, self.kind)
    }
}

impl CommonRepositoryError for RepositoryError {}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
