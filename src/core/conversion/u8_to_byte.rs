
use crate::executive::result_status::ResultStatus;
use crate::executive::future::TrackFuture;
use std::collections::HashMap;
use crate::executive::model::Model;
use crate::executive::value::Value;
use crate::executive::transmitter::*;
use crate::executive::treatment::Treatment;
use crate::executive::world::World;
use crate::logic::descriptor::{InputDescriptor, OutputDescriptor, FlowDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor};
use crate::logic::descriptor::identifier::core_identifier;
use std::sync::{Arc, Weak, RwLock};
use downcast_rs::DowncastSync;
use crate::logic::descriptor::CoreTreatmentDescriptor;

pub struct U8ToByte {

    world: Arc<World>,

    data_byte_transmitters: RwLock<Vec<Transmitter>>,
    data_u8_sender: Sender<u8>,
    data_u8_receiver: Receiver<u8>,

    auto_reference: RwLock<Weak<Self>>,
}

impl U8ToByte {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("conversion";"U8ToByte"),
                    Vec::new(),
                    HashMap::new(),
                    Vec::new(),
                    vec![InputDescriptor::new(
                        "data",
                        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::U8),
                        FlowDescriptor::Stream
                    )],
                    vec![OutputDescriptor::new(
                        "data",
                        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Byte),
                        FlowDescriptor::Stream
                    )],
                    U8ToByte::new,
                );

                rc_descriptor
            };
        }

        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Treatment> {
        let data_u8 = bounded(1048576);
        let treatment = Arc::new(Self {
            world,
            data_byte_transmitters: RwLock::new(Vec::new()),
            data_u8_sender: data_u8.0,
            data_u8_receiver: data_u8.1,
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }

    async fn convert(&self) -> ResultStatus {

        let inputs_to_fill = self.data_byte_transmitters.read().unwrap().clone();

        while let Ok(data) = self.data_u8_receiver.recv().await {

            for transmitter in &inputs_to_fill {
                match transmitter {
                    Transmitter::Byte(sender) => sender.send(data).await.unwrap(),
                    _ => panic!("Byte sender expected!")
                };
            }
        }

        for transmitter in inputs_to_fill {
            match transmitter {
                Transmitter::Byte(sender) => sender.close(),
                _ => panic!("Byte sender expected!")
            };
        }

        ResultStatus::default()
    }
}

impl Treatment for U8ToByte {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Self::descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {
        panic!("No parameter expected.")
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {
        panic!("No model expected.")
    }

    fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>) {
        
        match output_name {
            "data" => self.data_byte_transmitters.write().unwrap().extend(transmitter),
            _ => panic!("No output '{}' exists.", output_name)
        }
    }

    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {

        let mut hashmap = HashMap::new();

        hashmap.insert("data".to_string(), vec![Transmitter::U8(self.data_u8_sender.clone())]);

        hashmap
    }

    fn prepare(&self) -> Vec<TrackFuture> {

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future = Box::new(Box::pin(async move { auto_self.convert().await }));

        vec![future]
    }
    
}
