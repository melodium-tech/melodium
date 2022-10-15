
//! Provides generic collection for logical element types.

use std::collections::HashMap;
use std::sync::Arc;
use super::descriptor::IdentifiedDescriptor;
use super::descriptor::IdentifierDescriptor;

#[derive(Debug)]
pub struct Collection<T: IdentifiedDescriptor + Send + Sync + ?Sized> {
    descriptors: HashMap<IdentifierDescriptor, Arc<T>>,
}

impl<T: IdentifiedDescriptor + Send + Sync + ?Sized> Collection<T> {

    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new()
        }
    }

    pub fn identifiers(&self) -> Vec<IdentifierDescriptor> {
        self.descriptors.keys().map(|i| i.clone()).collect()
    }

    pub fn insert(&mut self, descriptor: &Arc<T>) {
        self.descriptors.insert(descriptor.identifier().clone(), Arc::clone(descriptor));
    }

    pub fn get(&self, id: &IdentifierDescriptor) -> Option<&Arc<T>> {
        self.descriptors.get(id)
    }

    pub fn get_tree_path(&self, steps: Vec<String>) -> Vec<IdentifierDescriptor> {
        self.descriptors.keys().filter(|id| {
            id.to_string().starts_with(&steps.join("/"))
        }).map(|id| id.clone()).collect()
    }
}

impl<T: IdentifiedDescriptor + Send + Sync + ?Sized> Clone for Collection<T> {
    
    fn clone(&self) -> Self {
        Self {
            descriptors: self.descriptors.iter().map(|(k,v)| (k.clone(), Arc::clone(&v))).collect()
        }
    }
}
