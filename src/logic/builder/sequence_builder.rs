
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::Builder;
use super::super::designer::SequenceDesigner;
use super::super::designer::TreatmentDesigner;
use super::super::designer::{ConnectionDesigner, ConnectionIODesigner};
use super::super::descriptor::parameterized::Parameterized;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::model::Model;
use super::super::super::executive::transmitter::Transmitter;
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

        // Esthablishing the order of creation of treatments.
        let mut ordered_treatments: Vec<Rc<RefCell<TreatmentDesigner>>> = self.designer.borrow().treatments().values().cloned().collect();
        ordered_treatments.sort_by(
            |a, b|
            a.borrow().partial_cmp(&b.borrow()).unwrap()
        );
        *self.ordered_treatments.write().unwrap() = ordered_treatments;

        None
    }

    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment) -> Option<HashMap<String, Transmitter>> {

        let mut treatments_outputs: HashMap<String, HashMap<String, Transmitter>> = HashMap::new();

        // Invoke all treatments builders
        for treatment in &*self.ordered_treatments.read().unwrap() {

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

            // Setup inputs

            //We get all connections that have the treatment as input (end point).
            let input_connections: Vec<Rc<RefCell<ConnectionDesigner>>> = self.designer.borrow().connections().iter().filter_map(
                |conn|
                if conn.borrow().input_treatment() == &Some(ConnectionIODesigner::Treatment(Rc::downgrade(treatment))) {
                    Some(Rc::clone(conn))
                }
                else {
                    None
                }
            ).collect();

            // We get all the inputs required by the treatment
            for input_connection in input_connections {

                let borrowed_connection = input_connection.borrow();
                match borrowed_connection.output_treatment().as_ref().unwrap() {
                    ConnectionIODesigner::Sequence() => {

                        remastered_environment.add_input(
                            &borrowed_connection.input_name().as_ref().unwrap(),
                            environment.get_input(borrowed_connection.output_name().as_ref().unwrap()).unwrap().clone()
                        );
                    },
                    ConnectionIODesigner::Treatment(output_treatment) => {

                        let rc_output_treatment = output_treatment.upgrade().unwrap();
                        let borrowed_output_treatment = rc_output_treatment.borrow();

                        remastered_environment.add_input(
                            &borrowed_connection.input_name().as_ref().unwrap(),
                            treatments_outputs.get(borrowed_output_treatment.name()).unwrap()
                                .get(borrowed_connection.output_name().as_ref().unwrap()).unwrap().clone()
                        );
                    },
                };
            }

            let treatment_outputs = borrowed_treatment.descriptor().builder().dynamic_build(&*remastered_environment).unwrap();
            treatments_outputs.insert(borrowed_treatment.name().to_string(), treatment_outputs);

        }


        // We get all connections that are to sequence output.
        let self_output_connections: Vec<Rc<RefCell<ConnectionDesigner>>> = self.designer.borrow().connections().iter().filter_map(
            |conn|
            match conn.borrow().input_treatment().as_ref().unwrap() {
                ConnectionIODesigner::Sequence() => Some(Rc::clone(conn)),
                _ => None,
            }
        ).collect();

        // We fill the 'Self' outputs that will be returned by the builder.
        let mut outputs: HashMap<String, Transmitter> = HashMap::new();
        for connection in self_output_connections {

            let borrowed_connection = connection.borrow();
            match borrowed_connection.output_treatment().as_ref().unwrap() {
                ConnectionIODesigner::Sequence() => {

                    outputs.insert(
                        borrowed_connection.input_name().as_ref().unwrap().to_string(),
                        environment.get_input(borrowed_connection.output_name().as_ref().unwrap()).unwrap().clone()
                    );
                },
                ConnectionIODesigner::Treatment(output_treatment) => {

                    let rc_output_treatment = output_treatment.upgrade().unwrap();
                    let borrowed_output_treatment = rc_output_treatment.borrow();

                    outputs.insert(
                        borrowed_connection.input_name().as_ref().unwrap().to_string(),
                        treatments_outputs.get(borrowed_output_treatment.name()).unwrap()
                            .get(borrowed_connection.output_name().as_ref().unwrap()).unwrap().clone()
                    );
                },
            };
        }

        Some(outputs)
    }
}


