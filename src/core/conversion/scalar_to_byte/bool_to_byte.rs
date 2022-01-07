
use super::super::super::prelude::*;

pub struct BoolToByte {

    world: Arc<World>,

    data_output_transmitters: RwLock<Vec<Transmitter>>,
    data_input_sender: Sender<bool>,
    data_input_receiver: Receiver<bool>,

    auto_reference: RwLock<Weak<Self>>,

}

impl BoolToByte {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("conversion";"BoolToByte"),
                    models![],
                    treatment_sources![],
                    vec![],
                    vec![
                        input!("value",Scalar,Bool,Stream)
                    ],
                    vec![
                        output!("data",Scalar,Byte,Stream)
                    ],
                    BoolToByte::new,
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
            data_output_transmitters: RwLock::new(Vec::new()),
            data_input_sender: data_input.0,
            data_input_receiver: data_input.1,
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }

    async fn to_bytes(&self) -> ResultStatus {

        let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();

        while let Ok(value) = self.data_input_receiver.recv().await {

            let byte = match value {
                true => 1,
                false => 0,
            };

            for transmitter in &inputs_to_fill {
                match transmitter {
                    Transmitter::Byte(sender) => sender.send(byte).await.unwrap(),
                    _ => panic!("{} sender expected!", std::any::type_name::<u8>())
                };
            }
        }

        for transmitter in inputs_to_fill {
            match transmitter {
                Transmitter::Byte(sender) => sender.close(),
                _ => panic!("{} sender expected!", std::any::type_name::<u8>())
            };
        }

        ResultStatus::default()
    }
}

impl Treatment for BoolToByte {

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
            "data" => self.data_output_transmitters.write().unwrap().extend(transmitter),
            _ => panic!("No output '{}' exists.", output_name)
        }
    }

    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {

        let mut hashmap = HashMap::new();

        hashmap.insert("value".to_string(), vec![Transmitter::Bool(self.data_input_sender.clone())]);

        hashmap
    }

    fn prepare(&self) -> Vec<TrackFuture> {

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future = Box::new(Box::pin(async move { auto_self.to_bytes().await }));

        vec![future]
    }
    
}
