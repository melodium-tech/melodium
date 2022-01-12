
use super::super::prelude::*;
use encoding_rs::*;

struct DecodeBytes {

    world: Arc<World>,

    encoding: RwLock<String>,

    decoder: RwLock<Option<Decoder>>,

    data_output_transmitters: RwLock<Vec<Transmitter>>,
    data_input_sender: Sender<u8>,
    data_input_receiver: Receiver<u8>,

    auto_reference: RwLock<Weak<Self>>,

}

impl DecodeBytes {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("text";"DecodeBytes"),
                    models![],
                    treatment_sources![],
                    vec![
                        parameter!("encoding",Scalar,String,Some(Value::String("utf-8".to_string())))
                    ],
                    vec![
                        input!("data",Scalar,Byte,Stream)
                    ],
                    vec![
                        output!("value",Scalar,String,Stream)
                    ],
                    DecodeBytes::new,
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
            encoding: RwLock::new(String::from("utf-8")),
            decoder: RwLock::new(None),
            data_output_transmitters: RwLock::new(Vec::new()),
            data_input_sender: data_input.0,
            data_input_receiver: data_input.1,
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }

    async fn decode(&self) -> ResultStatus {

        let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();

        const BUF_SIZE: usize = 4096;
        let input_buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

        // TODO continue there

        while let Ok(data) = self.data_input_receiver.recv().await {

            let output_string = data.to_string();

            for transmitter in &inputs_to_fill {
                match transmitter {
                    Transmitter::String(sender) => sender.send(output_string.clone()).await.unwrap(),
                    _ => panic!("{} sender expected!", std::any::type_name::<String>())
                };
            }
        }

        for transmitter in inputs_to_fill {
            match transmitter {
                Transmitter::String(sender) => sender.close(),
                _ => panic!("{} sender expected!", std::any::type_name::<String>())
            };
        }

        ResultStatus::default()
    }
}

impl Treatment for DecodeBytes {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Self::descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {
        
        match param {
            "encoding" => {
                match value {
                    Value::String(value) => *self.encoding.write().unwrap() = value.clone(),
                    _ => panic!("Unexpected value type for 'encoding'."),
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
            "value" => self.data_output_transmitters.write().unwrap().extend(transmitter),
            _ => panic!("No output '{}' exists.", output_name)
        }
    }

    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {

        let mut hashmap = HashMap::new();

        hashmap.insert("data".to_string(), vec![Transmitter::Byte(self.data_input_sender.clone())]);

        hashmap
    }

    fn prepare(&self) -> Vec<TrackFuture> {

        if let Some(encoding) = Encoding::for_label(self.encoding.read().unwrap().as_bytes()) {
            *self.decoder.write().unwrap() = Some(encoding.new_decoder());
        }
        else {
            *self.decoder.write().unwrap() = Some(UTF_8.new_decoder());
        }

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future = Box::new(Box::pin(async move { auto_self.decode().await }));

        vec![future]
    }
    
}