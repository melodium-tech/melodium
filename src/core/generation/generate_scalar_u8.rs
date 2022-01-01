
use crate::executive::future::TrackFuture;
use std::collections::HashMap;
use super::scalar_u8_generator::ScalarU8Generator;
use crate::executive::model::Model;
use crate::executive::value::Value;
use crate::executive::transmitter::Transmitter;
use crate::executive::treatment::Treatment;
use crate::executive::world::World;
use crate::logic::descriptor::{OutputDescriptor, FlowDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor};
use crate::logic::descriptor::identifier::core_identifier;
use std::sync::{Arc, Weak, RwLock};
use downcast_rs::DowncastSync;
use crate::logic::descriptor::CoreTreatmentDescriptor;

pub struct GenerateScalarU8 {

    world: Arc<World>,

    generator: RwLock<Option<Arc<ScalarU8Generator>>>,
    data_transmitters: RwLock<Vec<Transmitter>>,

    auto_reference: RwLock<Weak<Self>>,
}

impl GenerateScalarU8 {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {
                let mut source_from = HashMap::new();

                source_from.insert(ScalarU8Generator::descriptor(), vec!["data".to_string()]);

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("generation";"GenerateScalarU8"),
                    vec![("generator".to_string(), ScalarU8Generator::descriptor())],
                    source_from,
                    Vec::new(),
                    Vec::new(),
                    vec![OutputDescriptor::new(
                        "data",
                        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::U8),
                        FlowDescriptor::Stream
                    )],
                    GenerateScalarU8::new,
                );

                rc_descriptor
            };
        }

        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Treatment> {
        let treatment = Arc::new(Self {
            world,
            generator: RwLock::new(None),
            data_transmitters: RwLock::new(Vec::new()),
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }
}

impl Treatment for GenerateScalarU8 {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Self::descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {
        panic!("No parameter expected.")
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {

        match name {
            "generator" => *self.generator.write().unwrap() = Some(Arc::clone(&model).downcast_arc::<ScalarU8Generator>().unwrap()),
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
