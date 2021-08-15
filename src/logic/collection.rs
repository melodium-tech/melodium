
//! Provides generic collection for logical element types.

use std::collections::HashMap;
use std::sync::Arc;
use super::descriptor::IdentifiedDescriptor;
use super::descriptor::IdentifierDescriptor;

#[derive(Debug)]
pub struct Collection<T: IdentifiedDescriptor + ?Sized> {
    descriptors: HashMap<IdentifierDescriptor, Arc<T>>,
}

impl<T: IdentifiedDescriptor + ?Sized> Collection<T> {

    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new()
        }
    }

    pub fn insert(&mut self, descriptor: &Arc<T>) {
        self.descriptors.insert(descriptor.identifier().clone(), Arc::clone(descriptor));
    }

    pub fn get(&self, id: &IdentifierDescriptor) -> Option<&Arc<T>> {
        self.descriptors.get(id)
    }
}

impl<T: IdentifiedDescriptor + ?Sized> Clone for Collection<T> {
    
    fn clone(&self) -> Self {
        Self {
            descriptors: self.descriptors.into_iter().map(|(k,v)| (k, Arc::clone(&v))).collect()
        }
    }
}
