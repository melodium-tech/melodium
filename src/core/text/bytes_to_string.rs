
use super::super::prelude::*;
use encoding_rs::*;

struct DecodeBytes {

    world: Arc<World>,

    encoding: RwLock<String>,

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

        let encoding = Encoding::for_label(self.encoding.read().unwrap().as_bytes()).unwrap_or(UTF_8);
        let mut decoder = encoding.new_decoder();

        let mut finished = false;
        while !finished {

            let mut bytes = Vec::new();

            finished = true;
            while let Ok(data) = self.data_input_receiver.recv().await {

                bytes.push(data);

                if bytes.len() >= 2usize.pow(20) {
                    finished = false;
                    break;
                }
            }

            let mut result = String::with_capacity(bytes.len() * 2);

            decoder.decode_to_string(&bytes, &mut result, finished);

            for transmitter in &inputs_to_fill {
                match transmitter {
                    Transmitter::String(sender) => sender.send(result.clone()).await.unwrap(),
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

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future = Box::new(Box::pin(async move { auto_self.decode().await }));

        vec![future]
    }
    
}

pub fn register(c: &mut CollectionPool) {

    c.treatments.insert(&(DecodeBytes::descriptor() as Arc<dyn TreatmentDescriptor>));

}
