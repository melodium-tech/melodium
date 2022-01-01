
use crate::executive::future::TrackFuture;
use std::collections::HashMap;
use super::file_writer::FileWriterModel;
use crate::executive::model::{Model, ModelId};
use crate::executive::value::Value;
use crate::executive::transmitter::Transmitter;
use crate::executive::treatment::Treatment;
use crate::executive::world::World;
use crate::executive::environment::{ContextualEnvironment, GenesisEnvironment};
use crate::logic::builder::*;
use async_std::future::Future;
use crate::executive::result_status::ResultStatus;
use crate::logic::descriptor::{ParameterDescriptor, InputDescriptor, FlowDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor};
use crate::logic::descriptor::identifier::core_identifier;
use std::sync::{Arc, Weak, RwLock};
use crate::logic::error::LogicError;
use downcast_rs::DowncastSync;
use crate::logic::descriptor::CoreTreatmentDescriptor;

pub struct WriteFileTreatment {

    world: Arc<World>,

    file_writer: RwLock<Option<Arc<FileWriterModel>>>,
    data_transmitters: RwLock<Vec<Transmitter>>,

    auto_reference: RwLock<Weak<Self>>,
}

impl WriteFileTreatment {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("fs","direct";"WriteFile"),
                    vec![("writer".to_string(), FileWriterModel::descriptor())],
                    HashMap::new(),
                    Vec::new(),
                    vec![InputDescriptor::new(
                        "data",
                        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Byte),
                        FlowDescriptor::Stream
                    )],
                    Vec::new(),
                    WriteFileTreatment::new,
                );

                rc_descriptor
            };
        }

        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Treatment> {
        let treatment = Arc::new(Self {
            world,
            file_writer: RwLock::new(None),
            data_transmitters: RwLock::new(Vec::new()),
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }
}

impl Treatment for WriteFileTreatment {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Self::descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {
        panic!("No parameter expected.")
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {

        match name {
            "writer" => *self.file_writer.write().unwrap() = Some(Arc::clone(&model).downcast_arc::<FileWriterModel>().unwrap()),
            _ => panic!("No model '{}' expected.", name)
        }
    }

    fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>) {
        
        match output_name {
            _ => panic!("No output '{}' exists.", output_name)
        }
    }

    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {

        let writer_sender = self.file_writer.read().unwrap().as_ref().unwrap().writer().clone();

        let mut hashmap = HashMap::new();

        hashmap.insert("data".to_string(), vec![Transmitter::Byte(writer_sender)]);

        hashmap
    }

    fn prepare(&self) -> Vec<TrackFuture> {
        Vec::new()
    }
    
}
