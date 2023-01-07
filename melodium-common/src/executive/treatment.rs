use crate::descriptor::Treatment as TreatmentDescriptor;
use crate::executive::{Input, Output, Model, TrackFuture, Value};
use core::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;

pub trait Treatment : Debug + Sync + Send {
    fn descriptor(&self) -> Arc<dyn TreatmentDescriptor>;

    fn set_parameter(&self, param: &str, value: &Value);
    fn set_model(&self, name: &str, model: &Arc<dyn Model>);

    fn assign_input(&mut self, input_name: &str, transmitter: Box<dyn Input>);
    fn assign_output(&mut self, output_name: &str, transmitter: Box<dyn Output>);
    fn get_inputs(&self) -> HashMap<String, Box<dyn Input>>;
    fn set_output(&self, output_name: &str, transmitter: &Box<dyn Input>);

    fn prepare(&self) -> Vec<TrackFuture>;
}
