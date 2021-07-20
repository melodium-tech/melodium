
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
use super::Builder;
use super::super::descriptor::model::Model;
use super::super::descriptor::buildable::Buildable;
use super::super::designer::ModelDesigner;
use super::super::descriptor::parameterized::Parameterized;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::model::Model as ExecutiveModel;
use super::super::super::executive::transmitter::Transmitter;
use super::super::designer::value::Value;

#[derive(Debug)]
pub struct ConfiguredModelBuilder {
    designer: Rc<RefCell<ModelDesigner>>
}

impl ConfiguredModelBuilder {
    pub fn new(designer: &Rc<RefCell<ModelDesigner>>) -> Self {
        Self {
            designer: Rc::clone(designer)
        }
    }
}

impl Builder for ConfiguredModelBuilder {

    fn static_build(&self, environment: &dyn GenesisEnvironment) -> Option<Arc<dyn ExecutiveModel>> {

        let mut remastered_environment = environment.base();

        // We do assign default values (will be replaced if some other explicitly assigned)
        for (_, declared_parameter) in self.designer.borrow().descriptor().parameters() {

            if let Some(data) = declared_parameter.default() {
                remastered_environment.add_variable(declared_parameter.name(), data.clone());
            }
        }

        // Assigning explicit data
        for (_, parameter) in self.designer.borrow().parameters().iter() {

            let borrowed_param = parameter.borrow();

            let data = match borrowed_param.value().as_ref().unwrap() {
                Value::Raw(data) => data,
                Value::Variable(name) => {
                    environment.get_variable(&name).unwrap()
                },
                // Not possible in model to use context, should have been catcher by designed, aborting
                _ => panic!("Impossible data recoverage")
            };

            remastered_environment.add_variable(borrowed_param.name(), data.clone());
        }

        self.designer.borrow().descriptor().core_model().builder().static_build(&*remastered_environment)
    }

    fn dynamic_build(&self,  _environment: &dyn ContextualEnvironment) -> Option<HashMap<String, Transmitter>> {

        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }
}
