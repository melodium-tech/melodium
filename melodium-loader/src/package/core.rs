use crate::content::Content;
use crate::package::package::PackageTrait;
use crate::{Loader, PackageInfo};
use melodium_common::descriptor::{
    Collection, Identifier, IdentifierRequirement, LoadingError, LoadingResult,
    Package as CommonPackage, PackageRequirement,
};
use semver::Version;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Debug)]
pub struct CorePackage {
    package: Arc<dyn CommonPackage>,
    requirements: Vec<PackageRequirement>,
    embedded_collection: RwLock<Option<Collection>>,
    contents: RwLock<HashMap<String, Arc<Content>>>,
}

impl CorePackage {
    pub fn new(package: Arc<dyn CommonPackage>) -> Self {
        Self {
            requirements: package.requirements().iter().map(|pr| pr.clone()).collect(),
            package,
            embedded_collection: RwLock::new(None),
            contents: RwLock::new(HashMap::new()),
        }
    }

    fn insure_content(&self, designation: &str) -> LoadingResult<()> {
        match self.package.embedded().get(designation) {
            Some(data) => {
                if self.contents.read().unwrap().contains_key(designation) {
                    LoadingResult::new_success(())
                } else {
                    Content::new(
                        designation,
                        data,
                        self.version(),
                        &self
                            .requirements
                            .iter()
                            .map(|pkg_req| {
                                (pkg_req.package.clone(), pkg_req.version_requirement.clone())
                            })
                            .collect(),
                    )
                    .convert_failure_errors(|err| LoadingError::content_error(158, Arc::new(err)))
                    .and_then(|content| {
                        self.contents
                            .write()
                            .unwrap()
                            .insert(designation.to_string(), Arc::new(content));
                        LoadingResult::new_success(())
                    })
                }
            }
            None => LoadingResult::new_failure(LoadingError::not_found(
                159,
                designation.trim_end_matches(".mel").to_string(),
            )),
        }
    }

    fn all_contents(&self) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        let embedded = self.package.embedded();
        for (designation, _) in embedded {
            result = result.and_degrade_failure(self.insure_content(designation));
        }

        result
    }

    fn insure_loading(
        loader: &Loader,
        identifiers: Vec<IdentifierRequirement>,
    ) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for identifier in &identifiers {
            result = result.and_degrade_failure(
                loader
                    .get_with_load(identifier)
                    .and(LoadingResult::new_success(())),
            );
        }

        result
    }

    fn designation(identifier: &IdentifierRequirement) -> String {
        format!("{}.mel", identifier.path().join("/"))
    }
}

impl PackageInfo for CorePackage {
    fn name(&self) -> &str {
        self.package.name()
    }

    fn version(&self) -> &Version {
        self.package.version()
    }

    fn requirements(&self) -> &Vec<PackageRequirement> {
        &self.requirements
    }

    fn entrypoints(&self) -> &HashMap<String, Identifier> {
        static MAP: OnceLock<HashMap<String, Identifier>> = OnceLock::new();
        MAP.get_or_init(|| HashMap::new())
    }
}

impl PackageTrait for CorePackage {
    fn embedded_collection(&self, loader: &Loader) -> LoadingResult<Collection> {
        let mut embedded_collection = self.embedded_collection.write().unwrap();
        if let Some(collection) = &*embedded_collection {
            LoadingResult::new_success(collection.clone())
        } else {
            let result = self.package.collection(loader);
            if let Some(collection) = result.success() {
                *embedded_collection = Some(collection.clone());
            }
            result
        }
    }

    fn full_collection(&self, loader: &Loader) -> LoadingResult<Collection> {
        let mut results = LoadingResult::new_success(Collection::new());

        results.merge(self.all_contents());
        if results.is_failure() {
            return results;
        }

        let mut collection = if let Some(collection) =
            results.merge_degrade_failure(self.embedded_collection(loader))
        {
            collection
        } else {
            return results;
        };

        // Getting all needs of each content, while being sure no circular dependency occurs
        let mut all_needs = BTreeMap::new();
        for (designation, content) in self.contents.read().unwrap().iter() {
            let needs = content.require();

            'need: for need in &needs {
                let need_designation = Self::designation(need);
                if let Some(other_needs) = all_needs.get(&need_designation) {
                    for other_need in other_needs {
                        if &Self::designation(other_need) == designation {
                            results.merge_degrade_failure::<()>(LoadingResult::new_failure(
                                LoadingError::circular_reference(160, need.clone()),
                            ));
                            continue 'need;
                        }
                    }
                }
            }

            all_needs.insert(designation.clone(), needs);
        }

        let mut external_needs: Vec<IdentifierRequirement> = Vec::new();
        let mut internal_needs = Vec::new();
        for (designation_requester, designation_requested, ref need) in all_needs
            .into_iter()
            .map(|(designation, needs)| {
                needs
                    .into_iter()
                    .map(|need| (designation.clone(), Self::designation(&need), need))
                    .collect::<Vec<_>>()
            })
            .flatten()
        {
            if collection.get(need).is_none() {
                if need.root() != self.name() {
                    if !external_needs
                        .iter()
                        .any(|en| en.path() == need.path() && en.name() == need.name())
                    {
                        //let external_package_version_req = self.requirements.iter().find(|pr| &pr.package == need.root()).map(|pr| pr.version_requirement.clone()).unwrap_or_else(|| VersionReq::STAR);
                        //let identifier_requirement = IdentifierRequirement::new_with_identifier(external_package_version_req, &need);
                        external_needs.push(need.clone());
                    }
                } else {
                    // Knowing we don't have circular dependency, we can apply this logic
                    let requester_included = internal_needs.contains(&designation_requester);
                    let requested_included = internal_needs.contains(&designation_requested);
                    if !requester_included && !requested_included {
                        internal_needs.push(designation_requested);
                        internal_needs.push(designation_requester);
                    } else if requester_included && !requested_included {
                        let position = internal_needs
                            .iter()
                            .position(|d| d == &designation_requester)
                            .unwrap();
                        internal_needs.insert(position, designation_requested);
                    } else {
                        internal_needs.push(designation_requester);
                    }
                }
            }
        }

        for identifier_req in &external_needs {
            if let Some(entry) = results.merge_degrade_failure(loader.get_with_load(identifier_req))
            {
                collection.insert(entry);
            }
        }

        let contents = self.contents.read().unwrap();
        for designation in &internal_needs {
            if let Some(content) = contents.get(designation) {
                results.merge_degrade_failure(
                    content
                        .insert_descriptors(&mut collection)
                        .convert_failure_errors(|err| {
                            LoadingError::content_error(161, Arc::new(err))
                        }),
                );
            }
        }
        for (designation, content) in &*contents {
            if !internal_needs.contains(designation) {
                results.merge_degrade_failure(
                    content
                        .insert_descriptors(&mut collection)
                        .convert_failure_errors(|err| {
                            LoadingError::content_error(162, Arc::new(err))
                        }),
                );
            }
        }

        results.and(LoadingResult::new_success(collection))
    }

    fn all_identifiers(&self, loader: &Loader) -> LoadingResult<Vec<Identifier>> {
        let mut results = LoadingResult::new_success(Vec::new());

        results.merge(self.all_contents());
        if results.is_failure() {
            return results;
        }

        self.embedded_collection(loader).and_then(|collection| {
            let mut identifiers = collection.identifiers();
            identifiers.extend(
                self.contents
                    .read()
                    .unwrap()
                    .iter()
                    .map(|(_, content)| {
                        content
                            .provide()
                            .into_iter()
                            .map(|id| id.with_version(self.package.version()))
                            .collect::<Vec<_>>()
                    })
                    .flatten(),
            );
            LoadingResult::new_success(identifiers)
        })
    }

    fn element(
        &self,
        loader: &Loader,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Collection> {
        let mut result = LoadingResult::new_success(Collection::new());

        if let Some(collection) = result.merge_degrade_failure(self.embedded_collection(loader)) {
            if collection.get(identifier_requirement).is_some() {
                return result.and_degrade_failure(LoadingResult::new_success(collection));
            }
        }

        let designation = Self::designation(identifier_requirement);

        if let None = result.merge_degrade_failure(self.insure_content(&designation)) {
            return result.and_degrade_failure(LoadingResult::new_failure(
                LoadingError::not_found(250, identifier_requirement.to_string()),
            ));
        }

        let content = { self.contents.read().unwrap().get(&designation).cloned() };
        if let Some(content) = content {
            if let Ok(_guard) = content.try_lock() {
                let needs = content.require();
                result.merge_degrade_failure(Self::insure_loading(loader, needs));

                let mut collection = loader.collection().clone();
                result.merge_degrade_failure(
                    content
                        .insert_descriptors(&mut collection)
                        .convert_failure_errors(|err| {
                            LoadingError::content_error(163, Arc::new(err))
                        }),
                );

                result = result.and_degrade_failure(LoadingResult::new_success(collection));
            } else {
                result.merge_degrade_failure::<()>(LoadingResult::new_failure(
                    LoadingError::circular_reference(164, identifier_requirement.clone()),
                ));
            }
        } else {
            result.merge_degrade_failure::<()>(LoadingResult::new_failure(
                LoadingError::not_found(165, identifier_requirement.to_string()),
            ));
        }

        result
    }

    fn make_building(&self, collection: &Arc<Collection>) -> LoadingResult<()> {
        let contents = self.contents.read().unwrap();
        let mut result = LoadingResult::new_success(());
        for (_, content) in contents.iter() {
            result.merge_degrade_failure(
                content
                    .make_design(collection)
                    .convert_failure_errors(|err| LoadingError::content_error(166, Arc::new(err))),
            );
        }

        result
    }
}
