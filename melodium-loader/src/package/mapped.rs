use crate::compo::Compo;
use crate::content::Content;
use crate::package::package::PackageTrait;
use crate::{Loader, PackageInfo, LIB_ROOT_FILENAME};
use melodium_common::descriptor::{
    Collection, Identifier, IdentifierRequirement, LoadingError, LoadingResult, PackageRequirement,
    Version,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct MappedPackage {
    name: String,
    version: Version,
    requirements: Vec<PackageRequirement>,
    entrypoints: HashMap<String, Identifier>,
    contents: HashMap<String, Arc<Content>>,
}

impl MappedPackage {
    pub fn new(map: HashMap<String, Vec<u8>>) -> LoadingResult<Self> {
        let mut result = LoadingResult::new_success(());

        let composition;
        if let Some(compo) = map
            .get("Compo.toml")
            .map(|content| String::from_utf8(content.clone()).ok())
            .flatten()
        {
            if let Some(compo) = result.merge_degrade_failure(Compo::parse(&compo)) {
                composition = compo;
            } else {
                return result.and_degrade_failure(LoadingResult::new_failure(
                    LoadingError::mapping_format_error(
                        243,
                        "Invalid Compo.toml content".to_string(),
                    ),
                ));
            }
        } else {
            return result.and_degrade_failure(LoadingResult::new_failure(
                LoadingError::mapping_format_error(
                    244,
                    "Mapping does not contain 'Compo.toml' entry".to_string(),
                ),
            ));
        };
        let mut contents = HashMap::new();

        for (path, data) in map.iter() {
            if let Some(content) = result.merge_degrade_failure(
                Content::new(
                    &if path == LIB_ROOT_FILENAME {
                        composition.name.clone()
                    } else {
                        format!("{}/{}", composition.name, path,)
                    },
                    data,
                    &composition.version,
                    &composition
                        .requirements
                        .iter()
                        .map(|pkg_req| {
                            (pkg_req.package.clone(), pkg_req.version_requirement.clone())
                        })
                        .collect(),
                )
                .convert_failure_errors(|err| LoadingError::content_error(231, Arc::new(err))),
            ) {
                contents.insert(path.clone(), Arc::new(content));
            }
        }

        result.and(LoadingResult::new_success(Self {
            name: composition.name,
            entrypoints: composition.entrypoints,
            version: composition.version,
            requirements: composition.requirements,
            contents,
        }))
    }

    fn designation(identifier: &Identifier) -> String {
        if identifier.path().len() == 1 {
            LIB_ROOT_FILENAME.to_string()
        } else {
            format!(
                "{}.mel",
                identifier
                    .path()
                    .clone()
                    .into_iter()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("/")
            )
        }
    }

    fn insure_loading(
        loader: &Loader,
        identifiers: Vec<IdentifierRequirement>,
    ) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for identifier in &identifiers {
            result.merge_degrade_failure(loader.get_with_load(identifier));
        }

        result
    }
}

impl PackageInfo for MappedPackage {
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

impl PackageTrait for MappedPackage {
    fn embedded_collection(&self, _loader: &Loader) -> LoadingResult<Collection> {
        LoadingResult::new_success(Collection::new())
    }

    fn full_collection(&self, loader: &Loader) -> LoadingResult<Collection> {
        let mut collection = Collection::new();
        let mut result = LoadingResult::new_success(());
        if let Some(identifiers) = result.merge_degrade_failure(self.all_identifiers(loader)) {
            for identifier in &identifiers {
                let identifier_requirement = identifier.into();
                if collection.get(&identifier_requirement).is_none() {
                    if let Some(specific_collection) =
                        result.merge_degrade_failure(self.element(loader, &identifier_requirement))
                    {
                        for identifier in &specific_collection.identifiers() {
                            collection.insert(
                                specific_collection.get(&identifier.into()).unwrap().clone(),
                            );
                        }
                    }
                }
            }
        }

        result.and(LoadingResult::new_success(collection))
    }

    fn all_identifiers(&self, _loader: &Loader) -> LoadingResult<Vec<Identifier>> {
        let mut identifiers = Vec::new();
        self.contents.iter().for_each(|(_, content)| {
            identifiers.extend(
                content
                    .provide()
                    .into_iter()
                    .map(|id| id.with_version(&self.version))
                    .collect::<Vec<_>>(),
            )
        });

        LoadingResult::new_success(identifiers)
    }

    fn element(
        &self,
        loader: &Loader,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Collection> {
        let mut result = LoadingResult::new_success(Collection::new());
        let designation = Self::designation(&identifier_requirement.to_identifier());

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
                    LoadingError::circular_reference(234, identifier_requirement.clone()),
                ));
            }
        } else {
            result = result.and_degrade_failure(LoadingResult::new_failure(
                LoadingError::not_found(235, identifier_requirement.to_string()),
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
