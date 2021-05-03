
use std::fmt::Debug;
use std::rc::Rc;
use intertrait::CastFrom;
use super::identified::Identified;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::core_model::CoreModel;

pub trait Model: Identified + Parameterized + Buildable + CastFrom + Debug {
    fn is_core_model(&self) -> bool;
    fn core_model(&self) -> Rc<CoreModel>;
}