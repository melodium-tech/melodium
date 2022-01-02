
use crate::executive::result_status::ResultStatus;
use crate::executive::future::TrackFuture;
use std::sync::atomic::{Ordering, AtomicU8};
use std::collections::HashMap;
use crate::executive::model::Model;
use crate::executive::value::Value;
use crate::executive::transmitter::*;
use crate::executive::treatment::Treatment;
use crate::executive::world::World;
use crate::logic::descriptor::{ParameterDescriptor, InputDescriptor, OutputDescriptor, FlowDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor};
use crate::logic::descriptor::identifier::core_identifier;
use crate::logic::descriptor::datatype::datatype;
use crate::logic::descriptor::input::input;
use crate::logic::descriptor::output::output;
use crate::logic::descriptor::parameter::parameter;
use crate::logic::descriptor::core_treatment::models;
use std::sync::{Arc, Weak, RwLock};
use downcast_rs::DowncastSync;
use crate::logic::descriptor::CoreTreatmentDescriptor;

pub struct AddScalarU8 {

    world: Arc<World>,

    value: AtomicU8,

    data_output_transmitters: RwLock<Vec<Transmitter>>,
    data_input_sender: Sender<u8>,
    data_input_receiver: Receiver<u8>,

    auto_reference: RwLock<Weak<Self>>,
}


impl AddScalarU8 {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {

                let mut parameters = Vec::new();

                let value_parameter = ParameterDescriptor::new(
                    "value",
                    datatype!(Scalar, U8),
                    None
                );

                parameters.push(value_parameter);

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("arithmetic";"AddScalarU8"),
                    models![],
                    HashMap::new(),
                    vec![
                        parameter!("value",Scalar,U8,None)
                    ],
                    vec![
                        input!("data",Scalar,U8,Stream)
                    ],
                    vec![
                        output!("data",Scalar,U8,Stream)
                    ],
                    AddScalarU8::new,
                );

                rc_descriptor
            };
        }

        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Treatment> {
        let data_input = unbounded();
        let treatment = Arc::new(Self {
            world,
            value: AtomicU8::new(0),
            data_output_transmitters: RwLock::new(Vec::new()),
            data_input_sender: data_input.0,
            data_input_receiver: data_input.1,
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }

    async fn add(&self) -> ResultStatus {

        let value = self.value.load(Ordering::Relaxed);
        let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();

        while let Ok(data) = self.data_input_receiver.recv().await {

            let output_data = data + value;

            for transmitter in &inputs_to_fill {
                match transmitter {
                    Transmitter::U8(sender) => sender.send(output_data).await.unwrap(),
                    _ => panic!("u8 sender expected!")
                };
            }
        }

        for transmitter in inputs_to_fill {
            match transmitter {
                Transmitter::U8(sender) => sender.close(),
                _ => panic!("u8 sender expected!")
            };
        }

        ResultStatus::default()
    }
}

impl Treatment for AddScalarU8 {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Self::descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {
        
        match param {
            "value" => {
                match value {
                    Value::U8(value) => self.value.store(*value, Ordering::Relaxed),
                    _ => panic!("Unexpected value type for 'value'."),
                }
            },
            _ => panic!("No parameter '{}' exists.", param)
        }
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {
        panic!("No model expected.")
    }

    fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>) {
        
        match output_name {
            "data" => self.data_output_transmitters.write().unwrap().extend(transmitter),
            _ => panic!("No output '{}' exists.", output_name)
        }
    }

    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {

        let mut hashmap = HashMap::new();

        hashmap.insert("data".to_string(), vec![Transmitter::U8(self.data_input_sender.clone())]);

        hashmap
    }

    fn prepare(&self) -> Vec<TrackFuture> {

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future = Box::new(Box::pin(async move { auto_self.add().await }));

        vec![future]
    }
    
}
