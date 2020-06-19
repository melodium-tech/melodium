
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default)]
pub struct Reference<T> {
    pub name: String,
    pub reference: Option<Rc<RefCell<T>>>,
}

impl<T> Reference<T> {
    pub fn new(name: String) -> Self {

        Self {
            name,
            reference: None,
        }
    }
}