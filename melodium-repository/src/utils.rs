use crate::global::{Author, Package as PackageDetails, Tag, Type as TypeDetails};
use crate::technical::{Package, Type};
use cargo_metadata::{CargoOpt, DependencyKind, MetadataCommand};
use core::fmt::Display;
use melodium_common::descriptor::PackageRequirement;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Melodium(String),
    Metadata(cargo_metadata::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Melodium(str) => write!(f, "{str}"),
            Error::Metadata(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<cargo_metadata::Error> for Error {
    fn from(value: cargo_metadata::Error) -> Self {
        Error::Metadata(value)
    }
}

pub fn cargo_toml(
    package: &str,
    path: impl Into<PathBuf>,
) -> Result<(Package, PackageDetails), Error> {
    let metadata = MetadataCommand::new()
        .features(CargoOpt::SomeFeatures(vec!["real".into(), "plugin".into()]))
        .manifest_path(path)
        .exec()?;

    let rust_name = format!("{package}-mel");

    if let Some(rust_package) = metadata.packages.iter().find(|p| p.name == rust_name) {
        Ok((rust_package.into(), rust_package.into()))
    } else {
        Err(Error::Melodium(format!(
            "Package '{package}' not found (looking for '{rust_name}')"
        )))
    }
}

impl From<&cargo_metadata::Package> for Package {
    fn from(value: &cargo_metadata::Package) -> Self {
        Package {
            name: value
                .name
                .strip_suffix("-mel")
                .unwrap_or(&value.name)
                .to_string(),
            version: value.version.clone(),
            requirements: value
                .dependencies
                .iter()
                .filter_map(|dep| {
                    dep.name
                        .strip_suffix("-mel")
                        .filter(|_| dep.kind == DependencyKind::Normal)
                        .map(|name| PackageRequirement {
                            package: name.to_string(),
                            version_requirement: dep.req.clone(),
                        })
                })
                .collect(),

            r#type: Type::Compiled {
                crate_name: value.name.clone(),
                platforms: vec![],
            },
        }
    }
}

impl From<&cargo_metadata::Package> for PackageDetails {
    fn from(value: &cargo_metadata::Package) -> Self {
        PackageDetails {
            name: value
                .name
                .strip_suffix("-mel")
                .unwrap_or(&value.name)
                .to_string(),
            authors: value.authors.iter().map(|a| Author::new(a)).collect(),
            publication: chrono::Utc::now(),
            description: vec![(
                "en".to_string(),
                value.description.clone().unwrap_or_default(),
            )]
            .into_iter()
            .collect(),
            version: value.version.clone(),
            license: value.license.clone().unwrap_or_default(),
            homepage: value.homepage.clone(),
            repository: value.repository.clone(),
            r#type: TypeDetails::Compiled,
            tags: vec![Tag::Std],
        }
    }
}
