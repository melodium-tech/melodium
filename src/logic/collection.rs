
use std::collections::HashMap;
use std::rc::Rc;
use super::descriptor::IdentifiedDescriptor;
use super::descriptor::IdentifierDescriptor;

pub struct Collection<T: IdentifiedDescriptor + ?Sized> {
    descriptors: HashMap<IdentifierDescriptor, Rc<T>>,
}

impl<T: IdentifiedDescriptor + ?Sized> Collection<T> {

    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new()
        }
    }

    pub fn insert(&mut self, descriptor: &Rc<T>) {
        self.descriptors.insert(descriptor.identifier().clone(), Rc::clone(descriptor));
    }

    pub fn get(&self, id: &IdentifierDescriptor) -> Option<&Rc<T>> {
        self.descriptors.get(id)
    }
}