
use std::rc::Rc;
use super::identified::Identified;
use super::parameterized::Parameterized;
use super::core_model::CoreModel;

pub trait Model: Identified + Parameterized {
    fn is_core_model(&self) -> bool;
    fn core_model(&self) -> Rc<CoreModel>;
}