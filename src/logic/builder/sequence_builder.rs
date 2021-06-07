
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::Builder;
use super::super::designer::SequenceDesigner;
use super::super::descriptor::parameterized::Parameterized;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::model::Model;
use super::super::designer::value::Value;

#[derive(Debug)]
pub struct SequenceBuilder {
    designer: Rc<RefCell<SequenceDesigner>>,
    instancied_models: RwLock<HashMap<String, Arc<dyn Model>>>,
}

impl SequenceBuilder {
    pub fn new(designer: &Rc<RefCell<SequenceDesigner>>) -> Self {
        Self {
            designer: Rc::clone(designer),
            instancied_models: RwLock::new(HashMap::new()),
        }
    }
}

impl Builder for SequenceBuilder {

    fn static_build(&self, environment: &dyn GenesisEnvironment) -> Option<Arc<dyn Model>> {

        for (instanciation_name, model_instanciation) in self.designer.borrow().model_instanciations() {

            let mut remastered_environment = environment.base();

            for (_, parameter) in model_instanciation.borrow().parameters() {

                let borrowed_param = parameter.borrow();

                let data = match borrowed_param.value().as_ref().unwrap() {
                    Value::Raw(data) => data,
                    Value::Variable(name) => {
                        environment.get_variable(&name).unwrap()
                    },
                    // Not possible in model instanciation to use context, should have been catcher by designed, aborting
                    _ => panic!("Impossible data recoverage")
                };

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

            let instancied_model = model_instanciation.borrow().descriptor().builder().static_build(&*remastered_environment);
            self.instancied_models.write().unwrap().insert(instanciation_name.to_string(), instancied_model.unwrap());

        }

        None
    }

    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment) {

    }
}


