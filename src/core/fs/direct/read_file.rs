
use super::super::super::prelude::*;
use super::file_reader::FileReaderModel;

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
                    
                    models![
                        ("reader", FileReaderModel::descriptor())
                    ],
                    treatment_sources![
                        (FileReaderModel::descriptor(), "read")
                    ],
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
