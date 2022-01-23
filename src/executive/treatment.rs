
use std::fmt::Debug;
use crate::logic::descriptor::CoreTreatmentDescriptor;
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

    fn set_output(&self, output_name: &str, transmitter: Vec<Input>);
    fn get_inputs(&self) -> HashMap<String, Vec<Input>>;

    fn prepare(&self) -> Vec<TrackFuture>;
}

pub trait TreatmentImpl : Debug {
    fn process(&self);
}

#[derive(Debug)]
pub struct TreatmentHost {

    descriptor: Arc<CoreTreatmentDescriptor>,

    models: Mutex<HashMap<String, Arc<dyn Model>>>,
    parameters: Mutex<HashMap<String, Value>>,

    inputs: Mutex<HashMap<String, Input>>,
    outputs: Mutex<HashMap<String, Output>>,
}

impl TreatmentHost {

    pub fn new(descriptor: Arc<CoreTreatmentDescriptor>) -> Self {
        todo!()
    }
}
