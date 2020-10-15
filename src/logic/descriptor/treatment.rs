
use std::collections::HashMap;
use std::rc::Rc;
use super::identified::Identified;
use super::input::Input;
use super::output::Output;
use super::core_model::CoreModel;
use super::parameter::Parameter;
use super::requirement::Requirement;

pub trait Treatment: Identified {
    fn inputs(&self) -> &HashMap<String, Input>;
    fn outputs(&self) -> &HashMap<String, Output>;
    fn models(&self) -> &HashMap<String, Rc<CoreModel>>;
    fn parameters(&self) -> &HashMap<String, Parameter>;
    fn requirements(&self) -> &HashMap<String, Requirement>;
}
