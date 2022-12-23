
use std::sync::Arc;
use std::collections::HashMap;
use crate::descriptor::Treatment as TreatmentDescriptor;
use crate::executive::{Input, Model, TrackFuture, Value};

pub trait Treatment {

    fn descriptor(&self) -> Arc<dyn TreatmentDescriptor>;

    fn set_parameter(&self, param: &str, value: &Value);
    fn set_model(&self, name: &str, model: &Arc<dyn Model>);

    fn set_output(&self, output_name: &str, transmitter: &dyn Input);
    // TODO change for get_input() -> &dyn Input
    fn get_inputs(&self) -> HashMap<String, Box<dyn Input>>;

    fn prepare(&self) -> Vec<TrackFuture>;
}
