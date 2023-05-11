use crate::content::Content;
use crate::package::package::Package;
use crate::Loader;
use melodium_common::descriptor::{
    Collection, Identifier, LoadingError, LoadingResult, Package as CommonPackage,
};
use semver::Version;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct CorePackage {
    package: Box<dyn CommonPackage>,
    requirements: Vec<String>,
    embedded_collection: RwLock<Option<Collection>>,
    contents: RwLock<HashMap<String, Content>>,
}

impl CorePackage {
    pub fn new(package: Box<dyn CommonPackage>) -> Self {
        Self {
            requirements: package
                .requirements()
                .iter()
                .map(|s| s.to_string())
                .collect(),
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
                    Content::new(designation, data)
                        .convert_failure_errors(|err| {
                            LoadingError::content_error(158, Arc::new(err))
                        })
                        .and_then(|content| {
                            self.contents
                                .write()
                                .unwrap()
                                .insert(designation.to_string(), content);
                            LoadingResult::new_success(())
                        })
                }
            }
            None => {
                LoadingResult::new_failure(LoadingError::not_found(159, designation.to_string()))
            }
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

    fn insure_loading(loader: &Loader, identifiers: Vec<Identifier>) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for identifier in identifiers {
            result = result.and_degrade_failure(
                loader
                    .get_with_load(&identifier)
                    .and(LoadingResult::new_success(())),
            );
        }

        result
    }

    fn designation(identifier: &Identifier) -> String {
        format!("{}.mel", identifier.path().join("/"))
    }
}

impl Package for CorePackage {
    fn name(&self) -> &str {
        self.package.name()
    }

    fn version(&self) -> &Version {
        self.package.version()
    }

    fn requirements(&self) -> &Vec<String> {
        &self.requirements
    }

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
        let mut all_needs = HashMap::new();
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

        let mut external_needs = Vec::new();
        let mut internal_needs = Vec::new();
        for (designation_requester, designation_requested, need) in all_needs
            .into_iter()
            .map(|(designation, needs)| {
                needs
                    .into_iter()
                    .map(|need| (designation.clone(), Self::designation(&need), need))
                    .collect::<Vec<_>>()
            })
            .flatten()
        {
            if collection.get(&need).is_none() {
                if need.root() != self.name() {
                    if !external_needs.contains(&need) {
                        external_needs.push(need);
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

        for identifier in external_needs {
            if let Some(entry) = results.merge_degrade_failure(loader.get_with_load(&identifier)) {
                collection.insert(entry);
            }
        }

        let contents = self.contents.read().unwrap();
        for designation in &internal_needs {
            let content = contents.get(designation).unwrap();
            results.merge_degrade_failure(
                content
                    .insert_descriptors(&mut collection)
                    .convert_failure_errors(|err| LoadingError::content_error(161, Arc::new(err))),
            );
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

        results
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
                    .map(|(_, content)| content.provide())
                    .flatten(),
            );
            LoadingResult::new_success(identifiers)
        })
    }

    fn element(&self, loader: &Loader, identifier: &Identifier) -> LoadingResult<Collection> {
        let mut result = LoadingResult::new_success(Collection::new());

        if let Some(collection) = result.merge_degrade_failure(self.embedded_collection(loader)) {
            if collection.get(identifier).is_some() {
                return result.and_degrade_failure(LoadingResult::new_success(collection));
            }
        }

        let designation = Self::designation(identifier);

        if let None = result.merge_degrade_failure(self.insure_content(&designation)) {
            return result;
        }

        if let Some(content) = self.contents.read().unwrap().get(&designation) {
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
                    LoadingError::circular_reference(164, identifier.clone()),
                ));
            }
        } else {
            result.merge_degrade_failure::<()>(LoadingResult::new_failure(
                LoadingError::not_found(165, identifier.to_string()),
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
