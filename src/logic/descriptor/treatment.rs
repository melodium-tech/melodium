
use std::fmt::Debug;
use std::collections::HashMap;
use std::rc::Rc;
use intertrait::CastFrom;
use super::identified::Identified;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::input::Input;
use super::output::Output;
use super::core_model::CoreModel;
use super::requirement::Requirement;

pub trait Treatment: Identified + Parameterized + Buildable + CastFrom + Debug {
    fn inputs(&self) -> &HashMap<String, Input>;
    fn outputs(&self) -> &HashMap<String, Output>;
    fn models(&self) -> &HashMap<String, Rc<CoreModel>>;
    fn requirements(&self) -> &HashMap<String, Requirement>;
}
