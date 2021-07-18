
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::Builder;
use super::super::designer::SequenceDesigner;
use super::super::designer::TreatmentDesigner;
use super::super::descriptor::parameterized::Parameterized;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::model::Model;
use super::super::designer::value::Value;

#[derive(Debug)]
pub struct SequenceBuilder {
    designer: Rc<RefCell<SequenceDesigner>>,
    instancied_models: RwLock<HashMap<String, Arc<dyn Model>>>,
    ordered_treatments: RwLock<Vec<Rc<RefCell<TreatmentDesigner>>>>,
}

impl SequenceBuilder {
    pub fn new(designer: &Rc<RefCell<SequenceDesigner>>) -> Self {
        Self {
            designer: Rc::clone(designer),
            instancied_models: RwLock::new(HashMap::new()),
            ordered_treatments: RwLock::new(Vec::new()),
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
                    // Not possible in model instanciation to use context, should have been catched by designer, aborting
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

        // Invoke all treatments builders

        for (_, treatment) in self.designer.borrow().treatments() {

            let borrowed_treatment = treatment.borrow();
            let mut remastered_environment = environment.base();

            // Setup models
            for (model_treatment_name, _model) in borrowed_treatment.descriptor().models() {

                // model_treatment_name is the name of the model as seen by the treatment,
                // while model_sequence_name is the name of the model as it exists within the sequence.
                // Treatment[model_treatment_name = model_sequence_name]

                let model_sequence_name = borrowed_treatment.models().get(model_treatment_name).unwrap();

                let mut executive_model = None;
                
                if let Some(sequence_parameter_given_model) = environment.get_model(model_sequence_name) {
                    executive_model = Some(Arc::clone(sequence_parameter_given_model));
                }
                else {
                    let instancied_models = self.instancied_models.read().unwrap();
                    
                    if let Some(instancied_model) = instancied_models.get(model_sequence_name) {
                        executive_model = Some(Arc::clone(instancied_model));
                    }
                }

                if let Some(executive_model) = executive_model {

                    remastered_environment.add_model(model_treatment_name, executive_model);
                }
                else {
                    // We should have a model there, should have been catched by designer, aborting
                    panic!("Impossible model recoverage")
                }
            }

            // Setup parameters
            for (_, parameter) in borrowed_treatment.parameters() {

                let borrowed_param = parameter.borrow();

                let data = match borrowed_param.value().as_ref().unwrap() {
                    Value::Raw(data) => data,
                    Value::Variable(name) => {
                        environment.get_variable(&name).unwrap()
                    },
                    Value::Context((context, name)) => {
                        environment.get_context(context).unwrap().get_value(name).unwrap()
                    }
                };

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

        }

        // Create all connections


    }
}


