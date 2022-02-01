
use std::fmt::Debug;
use crate::logic::descriptor::{CoreTreatmentDescriptor, ParameterizedDescriptor, TreatmentDescriptor};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_std::future::Future;
use super::result_status::ResultStatus;
use super::future::TrackFuture;
use super::value::Value;
use super::model::Model;
use super::input::Input;
use super::output::Output;

pub trait Treatment{

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor>;

    fn set_parameter(&self, param: &str, value: &Value);
    fn set_model(&self, name: &str, model: &Arc<dyn Model>);

    fn set_output(&self, output_name: &str, transmitter: &Input);
    fn get_inputs(&self) -> HashMap<String, Input>;

    fn prepare(&self) -> Vec<TrackFuture>;
}

pub trait TreatmentImpl : Debug {
    fn prepare(&self) -> Vec<TrackFuture>;
}

#[derive(Debug)]
pub struct TreatmentHost<'a> {

    treatment_impl: &'a Box<dyn TreatmentImpl>,
    descriptor: Arc<CoreTreatmentDescriptor>,

    models: Mutex<HashMap<String, Arc<dyn Model>>>,
    parameters: Mutex<HashMap<String, Value>>,

    inputs: Mutex<HashMap<String, Input>>,
    outputs: Mutex<HashMap<String, Output>>,
}

impl<'a> TreatmentHost<'a> {

    pub fn new(treatment_impl: &'a Box<dyn TreatmentImpl>, descriptor: Arc<CoreTreatmentDescriptor>) -> Self {

        let parameters = descriptor.parameters().iter().filter_map(
            |(_, param)| {
                if let Some(default) = param.default() {
                    Some((param.name().to_string(), *default))
                }
                else {
                    None
                }
            }
        ).collect();

        let inputs = descriptor.inputs().iter().map(
            |(_, input)| {
                (input.name().to_string(), Input::new(input))
            }
        ).collect();

        let outputs = descriptor.outputs().iter().map(
            |(_, output)| {
                (output.name().to_string(), Output::new(output))
            }
        ).collect();

        Self {
            treatment_impl,
            descriptor,
            models: Mutex::new(HashMap::new()),
            parameters: Mutex::new(parameters),
            inputs: Mutex::new(inputs),
            outputs: Mutex::new(outputs),
        }
    }

    pub fn get_model(&self, model: &str) -> Arc<dyn Model> {
        Arc::clone(self.models.lock().unwrap().get(model).unwrap())
    }

    pub fn get_parameter(&self, param: &str) -> Value {
        self.parameters.lock().unwrap().get(param).unwrap().clone()
    }

    pub fn get_input(&self, name: &str) -> Input {
        self.inputs.lock().unwrap().get(name).unwrap().clone()
    }

    pub fn get_output(&self, name: &str) -> Output {
        self.outputs.lock().unwrap().get(name).unwrap().clone()
    }
}

impl Treatment for TreatmentHost<'_> {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Arc::clone(&self.descriptor)
    }

    fn set_parameter(&self, param: &str, value: &Value) {

        if let Some(param_descriptor) = self.descriptor.parameters().get(param) {

            if param_descriptor.datatype().is_compatible(&value) {

                self.parameters.lock().unwrap().insert(param.to_string(), *value);
            }
            else {
                panic!("Uncompatible value type for '{}'", param)
            }
        }
        else {
            panic!("Unknown parameter '{}'", param)
        }
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {

        if let Some(model_descriptor) = self.descriptor.models().get(name) {

            if *model_descriptor == model.descriptor() {

                self.models.lock().unwrap().insert(name.to_string(), Arc::clone(model));
            }
            else {
                panic!("Wrong model type for '{}'", name)
            }
        }
        else {
            panic!("Unknown model '{}'", name)
        }
    }

    fn set_output(&self, output_name: &str, transmitter: &Input) {

        if let Some(output) = self.outputs.lock().unwrap().get(output_name) {

            output.add_input(&transmitter);
        }
        else {
            panic!("Unknown output '{}'", output_name);
        }
    }

    fn get_inputs(&self) -> HashMap<String, Input> {

        self.inputs.lock().unwrap().clone()
    }

    fn prepare(&self) -> Vec<TrackFuture> {
        self.treatment_impl.prepare()
    }
}
