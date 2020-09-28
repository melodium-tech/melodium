
use std::rc::Rc;
use std::cell::RefCell;
use super::declared_parameter::DeclaredParameter;
use super::requirement::Requirement;

pub trait Parameter {
    
    /// Search for a declared parameter in the hosting element.
    fn find_declared_parameter(&self, name: & str) -> Option<Rc<RefCell<DeclaredParameter>>>;
    /// Search for a requirement in the hosting element.
    fn find_requirement(&self, _name: & str) -> Option<Rc<RefCell<Requirement>>> {
        None
    }
}
