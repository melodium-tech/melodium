
use super::super::prelude::*;
use super::tcp_listener::TcpListenerModel;

pub struct ReadTcpConnectionTreatment {

    world: Arc<World>,

    tcp_listener: RwLock<Option<Arc<TcpListenerModel>>>,
    data_transmitters: RwLock<Vec<Transmitter>>,

    auto_reference: RwLock<Weak<Self>>,
}

impl ReadTcpConnectionTreatment {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("net";"ReadTcpConnection"),
                    models![
                        ("listener", TcpListenerModel::descriptor())
                    ],
                    treatment_sources![
                        (TcpListenerModel::descriptor(), "connection")
                    ],
                    vec![],
                    vec![],
                    vec![
                        output!("data", Scalar, Byte, Stream)
                    ],
                    ReadTcpConnectionTreatment::new,
                );

                rc_descriptor
            };
        }

        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Treatment> {
        let treatment = Arc::new(Self {
            world,
            tcp_listener: RwLock::new(None),
            data_transmitters: RwLock::new(Vec::new()),
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }
}

impl Treatment for ReadTcpConnectionTreatment {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Self::descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {
        panic!("No parameter expected.")
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {

        match name {
            "listener" => *self.tcp_listener.write().unwrap() = Some(Arc::clone(&model).downcast_arc::<TcpListenerModel>().unwrap()),
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
