use crate::compo::Compo;
use crate::content::Content;
use crate::package::package::PackageTrait;
use crate::{Loader, PackageInfo};
use bzip2_rs::DecoderReader;
use melodium_common::descriptor::{
    Collection, Identifier, LoadingError, LoadingResult, PackageRequirement, Version,
};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tar::Archive;

#[derive(Debug)]
pub struct JeuPackage {
    name: String,
    version: Version,
    requirements: Vec<PackageRequirement>,
    entrypoints: HashMap<String, Identifier>,
    contents: HashMap<PathBuf, Content>,
}

impl JeuPackage {
    pub fn new<R: Read>(mut reader: R) -> LoadingResult<Self> {
        let mut result = LoadingResult::new_success(());

        if let None = result.merge_degrade_failure(Self::skip_heading(&mut reader)) {
            return Err(result.failure().unwrap().clone()).into();
        }

        let mut archive = Archive::new(DecoderReader::new(reader));

        let mut composition = None;
        let mut contents = HashMap::new();

        for entry in match archive.entries() {
            Ok(entries) => entries,
            Err(err) => {
                return result.and_degrade_failure(LoadingResult::new_failure(
                    LoadingError::jeu_format_error(225, err.to_string()),
                ))
            }
        } {
            match entry {
                Ok(mut entry) => match entry.path().map(|path| path.to_path_buf()) {
                    Ok(path) => {
                        if composition.is_none() && path.to_string_lossy() == "Compo.toml" {
                            let mut compo = String::new();
                            match entry.read_to_string(&mut compo) {
                                Ok(_) => {
                                    if let Some(compo) =
                                        result.merge_degrade_failure(Compo::parse(&compo))
                                    {
                                        composition = Some(compo);
                                    }
                                }
                                Err(err) => {
                                    result = result.and_degrade_failure(LoadingResult::new_failure(
                                        LoadingError::jeu_format_error(229, err.to_string()),
                                    ))
                                }
                            }
                        } else if let Some(composition) = &composition {
                            let mut content = Vec::new();
                            match entry.read_to_end(&mut content) {
                                Ok(_) => {
                                    if let Some(content) = result.merge_degrade_failure(
                                        Content::new(
                                            &format!(
                                                "{}/{}",
                                                composition.name,
                                                path.to_string_lossy()
                                            ),
                                            &content,
                                        )
                                        .convert_failure_errors(|err| {
                                            LoadingError::content_error(231, Arc::new(err))
                                        }),
                                    ) {
                                        contents.insert(path.to_path_buf(), content);
                                    }
                                }
                                Err(err) => {
                                    result = result.and_degrade_failure(LoadingResult::new_failure(
                                        LoadingError::jeu_format_error(230, err.to_string()),
                                    ))
                                }
                            }
                        } else {
                            result = result.and_degrade_failure(LoadingResult::new_failure(
                                LoadingError::jeu_format_error(
                                    231,
                                    format!(
                                        "Content '{}' appearing before 'Compo.toml'",
                                        path.to_string_lossy()
                                    ),
                                ),
                            ));
                        }
                    }
                    Err(err) => {
                        result = result.and_degrade_failure(LoadingResult::new_failure(
                            LoadingError::jeu_format_error(228, err.to_string()),
                        ))
                    }
                },
                Err(err) => {
                    result = result.and_degrade_failure(LoadingResult::new_failure(
                        LoadingError::jeu_format_error(226, err.to_string()),
                    ))
                }
            }
        }

        if let Some(compo) = composition {
            result.and(LoadingResult::new_success(Self {
                name: compo.name,
                entrypoints: compo.entrypoints,
                version: compo.version,
                requirements: compo.requirements,
                contents,
            }))
        } else {
            result.and_degrade_failure(LoadingResult::new_failure(LoadingError::jeu_format_error(
                232,
                "Jeu file does not contain 'Compo.toml' entry".to_string(),
            )))
        }
    }

    fn skip_heading<R: Read>(reader: &mut R) -> LoadingResult<()> {
        let mut encountered_lf = 0;

        let mut byte: u8 = 0;
        while encountered_lf < 2 {
            if let Ok(_) = reader.read_exact(std::slice::from_mut(&mut byte)) {
                if byte == 0x0A {
                    encountered_lf += 1;
                }
            } else {
                return LoadingResult::new_failure(LoadingError::jeu_format_error(
                    224,
                    "Data ended too early".to_string(),
                ));
            }
        }

        LoadingResult::new_success(())
    }

    fn designation(identifier: &Identifier) -> PathBuf {
        PathBuf::from(format!(
            "{}.mel",
            identifier
                .path()
                .clone()
                .into_iter()
                .skip(1)
                .collect::<Vec<_>>()
                .join("/")
        ))
    }

    fn insure_loading(loader: &Loader, identifiers: Vec<Identifier>) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for identifier in identifiers {
            result.merge_degrade_failure(loader.get_with_load(&identifier));
        }

        result
    }

    #[cfg(feature = "filesystem")]
    pub fn extract<R: Read>(mut reader: R, output: &std::path::Path) -> std::io::Result<()> {
        if Self::skip_heading(&mut reader).is_failure() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Input data are not Jeu format",
            ));
        }

        let mut archive = Archive::new(DecoderReader::new(reader));
        archive.unpack(&output)
    }

    #[cfg(feature = "filesystem")]
    pub fn build<W: std::io::Write>(
        fs_package: &super::FsPackage,
        mut writer: std::io::BufWriter<W>,
    ) -> std::io::Result<()> {
        use std::io::Write;

        fs_package.all_contents();

        writer.write_all(b"#!/usr/bin/env melodium\n#! version 0\n")?;

        let mut tar_data = Vec::new();

        {
            let mut tar = tar::Builder::new(&mut tar_data);

            let base_path = fs_package.path().to_path_buf();
            {
                let mut compo = std::fs::File::open({
                    let mut path = base_path.clone();
                    path.push("Compo.toml");
                    path
                })?;
                tar.append_file("Compo.toml", &mut compo)?;
            }

            for content_relative_path in fs_package.contents().keys() {
                let content_full_path = {
                    let mut path = base_path.clone();
                    path.push(content_relative_path);
                    path
                };

                let mut content = std::fs::File::open(content_full_path)?;
                tar.append_file(content_relative_path, &mut content)?;
            }
        }

        banzai::encode(&*tar_data, writer, 9).map(|_| ())
    }
}

impl PackageInfo for JeuPackage {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &semver::Version {
        &self.version
    }

    fn requirements(&self) -> &Vec<PackageRequirement> {
        &self.requirements
    }

    fn entrypoints(&self) -> &HashMap<String, Identifier> {
        &self.entrypoints
    }
}

impl PackageTrait for JeuPackage {
    fn embedded_collection(&self, _loader: &Loader) -> LoadingResult<Collection> {
        LoadingResult::new_success(Collection::new())
    }

    fn full_collection(&self, loader: &Loader) -> LoadingResult<Collection> {
        let mut collection = Collection::new();
        let mut result = LoadingResult::new_success(());
        if let Some(identifiers) = result.merge_degrade_failure(self.all_identifiers(loader)) {
            for identifier in identifiers {
                if collection.get(&identifier).is_none() {
                    if let Some(specific_collection) =
                        result.merge_degrade_failure(self.element(loader, &identifier))
                    {
                        for identifier in &specific_collection.identifiers() {
                            collection.insert(specific_collection.get(identifier).unwrap().clone());
                        }
                    }
                }
            }
        }

        result.and(LoadingResult::new_success(collection))
    }

    fn all_identifiers(&self, _loader: &Loader) -> LoadingResult<Vec<Identifier>> {
        let mut identifiers = Vec::new();
        self.contents
            .iter()
            .for_each(|(_, content)| identifiers.extend(content.provide()));

        LoadingResult::new_success(identifiers)
    }

    fn element(&self, loader: &Loader, identifier: &Identifier) -> LoadingResult<Collection> {
        let mut result = LoadingResult::new_success(Collection::new());
        let designation = Self::designation(identifier);

        if let Some(content) = self.contents.get(&designation) {
            if let Ok(_guard) = content.try_lock() {
                let needs = content.require();
                result.merge_degrade_failure(Self::insure_loading(loader, needs));

                let mut collection = loader.collection().clone();
                result.merge_degrade_failure(
                    content
                        .insert_descriptors(&mut collection)
                        .convert_failure_errors(|err| {
                            LoadingError::content_error(233, Arc::new(err))
                        }),
                );

                result = result.and_degrade_failure(LoadingResult::new_success(collection));
            } else {
                result = result.and_degrade_failure(LoadingResult::new_failure(
                    LoadingError::circular_reference(234, identifier.clone()),
                ));
            }
        } else {
            result = result.and_degrade_failure(LoadingResult::new_failure(
                LoadingError::not_found(235, identifier.to_string()),
            ));
        }

        result
    }

    fn make_building(&self, collection: &Arc<Collection>) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for (_, content) in self.contents.iter() {
            result.merge_degrade_failure(
                content
                    .make_design(collection)
                    .convert_failure_errors(|err| LoadingError::content_error(236, Arc::new(err))),
            );
        }

        result
    }
}
