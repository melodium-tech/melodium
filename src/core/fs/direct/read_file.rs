
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
use crate::logic::descriptor::{ParameterDescriptor, OutputDescriptor, FlowDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor, BuildableDescriptor};
use crate::logic::descriptor::identifier::core_identifier;
use std::sync::{Arc, Weak, RwLock};
use crate::logic::error::LogicError;
use downcast_rs::DowncastSync;
use crate::logic::descriptor::CoreTreatmentDescriptor;

pub struct ReadFileTreatment {

    world: Arc<World>,

    file_reader: RwLock<Option<Arc<FileReaderModel>>>,
    data_transmitters: RwLock<Vec<Transmitter>>,

    auto_reference: RwLock<Weak<Self>>,
}

impl ReadFileTreatment {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {
                let mut source_from = HashMap::new();

                source_from.insert(FileReaderModel::descriptor(), vec!["read".to_string()]);

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("fs","direct";"ReadFile"),
                        
                    vec![("reader".to_string(), FileReaderModel::descriptor())],
                    source_from,
                    Vec::new(),
                    Vec::new(),
                    vec![OutputDescriptor::new(
                        "data",
                        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Byte),
                        FlowDescriptor::Stream
                    )],
                    ReadFileTreatment::new,
                );

                rc_descriptor
            };
        }

        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Treatment> {
        let treatment = Arc::new(Self {
            world,
            file_reader: RwLock::new(None),
            data_transmitters: RwLock::new(Vec::new()),
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }
}

impl Treatment for ReadFileTreatment {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Self::descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {
        panic!("No parameter expected.")
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {

        match name {
            "reader" => *self.file_reader.write().unwrap() = Some(Arc::clone(&model).downcast_arc::<FileReaderModel>().unwrap()),
            _ => panic!("No model '{}' expected.", name)
        }
    }

    fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>) {
        
        match output_name {
            "data" => self.data_transmitters.write().unwrap().extend(transmitter),
            _ => panic!("No output '{}' exists.", output_name)
        }
    }

    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {

        let mut hashmap = HashMap::new();

        hashmap.insert("data".to_string(), self.data_transmitters.read().unwrap().clone());

        hashmap
    }

    fn prepare(&self) -> Vec<TrackFuture> {
        Vec::new()
    }
    
}
