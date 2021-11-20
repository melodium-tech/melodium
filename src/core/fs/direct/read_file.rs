
use crate::executive::future::TrackFuture;
use std::collections::HashMap;
use super::file_reader::FileReaderModel;
use crate::executive::model::{Model, ModelId};
use crate::executive::value::Value;
use crate::executive::transmitter::Transmitter;
use crate::executive::treatment::Treatment;
use crate::executive::world::World;
use crate::executive::environment::{ContextualEnvironment, GenesisEnvironment};
use crate::logic::builder::*;
use async_std::future::Future;
use crate::executive::result_status::ResultStatus;
use crate::logic::descriptor::{ParameterDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor};
use std::sync::{Arc, Weak, RwLock};
use crate::logic::error::LogicError;

pub struct ReadFileTreatment {

}

impl ReadFileTreatment {

    pub fn new() -> Arc<Self> {
        Arc::new(Self{})
    }
}

impl Treatment for ReadFileTreatment {

    fn set_parameter(&self, param: &str, value: &Value) {
        todo!()
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {
        todo!()
    }

    fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>) {
        todo!()
    }

    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {
        todo!()
    }

    fn prepare(&self) -> Vec<TrackFuture> {
        todo!()
    }
    
}
