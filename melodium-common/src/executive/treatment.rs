use crate::descriptor::{DataType, Treatment as TreatmentDescriptor};
use crate::executive::{Input, Model, Output, TrackFuture, Value};
use core::fmt::Debug;
use std::sync::Arc;

pub trait Treatment: Debug + Sync + Send {
    fn descriptor(&self) -> Arc<dyn TreatmentDescriptor>;

    fn set_generic(&self, generic: &str, data_type: DataType);
    fn set_parameter(&self, param: &str, value: Value);
    fn set_model(&self, name: &str, model: Arc<dyn Model>);

    fn assign_input(&self, input_name: &str, transmitter: Box<dyn Input>);
    fn assign_output(&self, output_name: &str, transmitter: Box<dyn Output>);

    fn prepare(&self, track_id: usize) -> Vec<TrackFuture>;
}
