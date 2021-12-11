
use crate::logic::descriptor::CoreTreatmentDescriptor;
use std::collections::HashMap;
use std::sync::Arc;
use async_std::future::Future;
use super::result_status::ResultStatus;
use super::transmitter::Transmitter;
use super::future::TrackFuture;
use super::value::Value;
use super::model::Model;

pub trait Treatment{

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor>;

    fn set_parameter(&self, param: &str, value: &Value);
    fn set_model(&self, name: &str, model: &Arc<dyn Model>);

    fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>);
    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>>;

    fn prepare(&self) -> Vec<TrackFuture>;
}
