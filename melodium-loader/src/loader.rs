
use melodium_common::descriptor::{Collection, Context, Entry, Function, Identifier, Loader as LoaderTrait, LoadingError, Model, Package, Treatment};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Loader {
    collection: RwLock<Collection>,
    packages: RwLock<HashMap<String, Arc<dyn Package>>>,
}

impl Loader {

    pub fn new(packages: Vec<Arc<dyn Package>>) -> Self {
        Self {
            collection: RwLock::new(Collection::new()),
            packages: RwLock::new(packages.into_iter().map(|p| (p.name().to_string(), p)).collect()),
        }
    }
}

impl LoaderTrait for Loader {
    fn load_context(&self, identifier: &Identifier) -> Result<Arc<Context>, LoadingError> {
        todo!()
    }

    fn load_function(&self, identifier: &Identifier) -> Result<Arc<dyn Function>, LoadingError> {
        todo!()
    }

    fn load_model(&self, identifier: &Identifier) -> Result<Arc<dyn Model>, LoadingError> {
        todo!()
    }

    fn load_treatment(&self, identifier: &Identifier) -> Result<Arc<dyn Treatment>, LoadingError> {
        todo!()
    }
}
