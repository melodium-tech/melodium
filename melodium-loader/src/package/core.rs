use crate::content::{Content, ContentError};
use crate::package::package::Package;
use crate::Loader;
use melodium_common::descriptor::{Collection, Identifier, LoadingError, Package as CommonPackage};
use semver::Version;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct CorePackage {
    package: Box<dyn CommonPackage>,
    requirements: Vec<String>,
    embedded_collection: RwLock<Option<Collection>>,
    contents: RwLock<HashMap<String, Content>>,
    errors: RwLock<Vec<ContentError>>,
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
            errors: RwLock::new(Vec::new()),
        }
    }

    fn insure_content(&self, designation: &str) -> Result<(), LoadingError> {
        match self.package.embedded().get(designation) {
            Some(data) => {
                if self.contents.read().unwrap().contains_key(designation) {
                    Ok(())
                } else {
                    let result_content = Content::new(designation, data);

                    match result_content {
                        Ok(content) => {
                            self.contents
                                .write()
                                .unwrap()
                                .insert(designation.to_string(), content);
                            Ok(())
                        }
                        Err(error) => {
                            self.errors.write().unwrap().push(error);
                            Err(LoadingError::NotFound(0))
                        }
                    }
                }
            }
            None => Err(LoadingError::NotFound(1)),
        }
    }

    fn all_contents(&self) -> Result<(), LoadingError> {
        let mut error = None;
        let embedded = self.package.embedded();
        for (designation, _) in embedded {
            if let Err(e) = self.insure_content(designation) {
                error = Some(e);
            }
        }

        error.map_or(Ok(()), |e| Err(e))
    }

    fn insure_loading(loader: &Loader, identifiers: Vec<Identifier>) -> Result<(), LoadingError> {
        for identifier in identifiers {
            loader.get_with_load(&identifier)?;
        }

        Ok(())
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

    fn embedded_collection(&self, loader: &Loader) -> Result<Collection, LoadingError> {
        let mut embedded_collection = self.embedded_collection.write().unwrap();
        if let Some(collection) = &*embedded_collection {
            Ok(collection.clone())
        } else {
            let collection = self.package.collection(loader)?;
            *embedded_collection = Some(collection.clone());
            Ok(collection)
        }
    }

    fn full_collection(&self, loader: &Loader) -> Result<Collection, LoadingError> {
        self.all_contents()?;

        let mut collection = self.embedded_collection(loader)?;

        // Getting all needs of each content, while being sure no circular dependency occurs
        let mut all_needs = HashMap::new();
        for (designation, content) in self.contents.read().unwrap().iter() {
            let needs = content.require();

            for need in &needs {
                let need_designation = Self::designation(need);
                if let Some(other_needs) = all_needs.get(&need_designation) {
                    for other_need in other_needs {
                        if &Self::designation(other_need) == designation {
                            return Err(LoadingError::CircularReference);
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
            collection.insert(loader.get_with_load(&identifier)?);
        }

        let contents = self.contents.read().unwrap();
        let mut content_error = false;
        for designation in &internal_needs {
            let content = contents.get(designation).unwrap();
            if let Err(error) = content.insert_descriptors(&mut collection) {
                self.errors.write().unwrap().push(error);
                content_error = true;
            }
        }
        for (designation, content) in &*contents {
            if !internal_needs.contains(designation) {
                if let Err(error) = content.insert_descriptors(&mut collection) {
                    self.errors.write().unwrap().push(error);
                    content_error = true;
                }
            }
        }

        if content_error {
            Err(LoadingError::ContentError)
        } else {
            Ok(collection)
        }
    }

    fn all_identifiers(&self, loader: &Loader) -> Result<Vec<Identifier>, LoadingError> {
        self.all_contents()?;

        let mut identifiers = self.embedded_collection(loader)?.identifiers();
        identifiers.extend(
            self.contents
                .read()
                .unwrap()
                .iter()
                .map(|(_, content)| content.provide())
                .flatten(),
        );

        Ok(identifiers)
    }

    fn element(
        &self,
        loader: &Loader,
        identifier: &Identifier,
    ) -> Result<Collection, LoadingError> {
        if let Some(_) = self.embedded_collection(loader)?.get(identifier) {
            return Ok(self.embedded_collection(loader)?);
        }

        let designation = Self::designation(identifier);
        self.insure_content(&designation)?;

        if let Some(content) = self.contents.read().unwrap().get(&designation) {
            if let Ok(_guard) = content.try_lock() {
                let needs = content.require();
                Self::insure_loading(loader, needs)?;

                let mut collection = loader.collection().clone();
                match content.insert_descriptors(&mut collection) {
                    Ok(()) => Ok(collection),
                    Err(error) => {
                        self.errors.write().unwrap().push(error);
                        Err(LoadingError::ContentError)
                    }
                }
            } else {
                Err(LoadingError::CircularReference)
            }
        } else {
            Err(LoadingError::NotFound(2))
        }
    }

    fn make_building(&self, collection: &Arc<Collection>) -> Result<(), LoadingError> {
        let contents = self.contents.read().unwrap();
        let mut content_error = false;
        for (_, content) in contents.iter() {
            if let Err(error) = content.make_design(collection) {
                self.errors.write().unwrap().push(error);
                content_error = true;
            }
        }

        if content_error {
            Err(LoadingError::ContentError)
        } else {
            Ok(())
        }
    }

    fn errors(&self) -> Vec<ContentError> {
        self.errors.read().unwrap().clone()
    }
}
