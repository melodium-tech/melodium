
use crate::executive::future::TrackFuture;
use std::collections::HashMap;
use super::tcp_listener::TcpListenerModel;
use async_std::net::TcpStream;
use async_std::io::BufWriter;
use async_std::prelude::*;
use crate::executive::model::{Model, ModelId};
use crate::executive::value::Value;
use crate::executive::transmitter::*;
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

pub struct WriteTcpConnectionTreatment {

    world: Arc<World>,

    to_ip: RwLock<Option<String>>,
    to_port: RwLock<Option<u16>>,

    tcp_listener: RwLock<Option<Arc<TcpListenerModel>>>,
    tcp_stream: RwLock<Option<TcpStream>>,
    data_input_sender: Sender<u8>,
    data_input_receiver: Receiver<u8>,

    auto_reference: RwLock<Weak<Self>>,
}

impl WriteTcpConnectionTreatment {

    pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {

                let mut parameters = Vec::new();

                let ip_parameter = ParameterDescriptor::new(
                    "ip",
                    DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::String),
                    None
                );

                parameters.push(ip_parameter);

                let port_parameter = ParameterDescriptor::new(
                    "port",
                    DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::U16),
                    None
                );

                parameters.push(port_parameter);

                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("net";"WriteTcpConnection"),
                    vec![("listener".to_string(), TcpListenerModel::descriptor())],
                    HashMap::new(),
                    parameters,
                    vec![InputDescriptor::new(
                        "data",
                        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Byte),
                        FlowDescriptor::Stream
                    )],
                    Vec::new(),
                    WriteTcpConnectionTreatment::new,
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
            to_ip: RwLock::new(None),
            to_port: RwLock::new(None),
            tcp_listener: RwLock::new(None),
            tcp_stream: RwLock::new(None),
            data_input_sender: data_input.0,
            data_input_receiver: data_input.1,
            auto_reference: RwLock::new(Weak::new()),
        });

        *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);

        treatment
    }

    async fn tcp_write(&self) -> ResultStatus {

        let stream = self.tcp_stream.read().unwrap().as_ref().unwrap().clone();
        let mut writer = BufWriter::with_capacity(1024, stream);

        while let Ok(data) = self.data_input_receiver.recv().await {

            if let Err(write_err) = writer.write(&[data]).await {
                // Todo handle error
                panic!("Writing error: {}", write_err)
            }
        }

        if let Err(write_err) = writer.flush().await {

            // Todo handle error
            panic!("Writing (flush) error: {}", write_err)
        }

        ResultStatus::default()
    }
}

impl Treatment for WriteTcpConnectionTreatment {

    fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
        Self::descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {

        match param {
            "ip" => {
                match value {
                    Value::String(ip) => *self.to_ip.write().unwrap() = Some(ip.clone()),
                    _ => panic!("Unexpected value type for 'ip'."),
                }
            },
            "port" => {
                match value {
                    Value::U16(port) => *self.to_port.write().unwrap() = Some(*port),
                    _ => panic!("Unexpected value type for 'port'."),
                }
            },
            _ => panic!("No parameter '{}' exists.", param)
        }
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {

        match name {
            "listener" => *self.tcp_listener.write().unwrap() = Some(Arc::clone(&model).downcast_arc::<TcpListenerModel>().unwrap()),
            _ => panic!("No model '{}' expected.", name)
        }
    }

    fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>) {
        
        match output_name {
            _ => panic!("No output '{}' exists.", output_name)
        }
    }

    fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {

        let mut hashmap = HashMap::new();

        hashmap.insert("data".to_string(), vec![Transmitter::Byte(self.data_input_sender.clone())]);

        hashmap
    }

    fn prepare(&self) -> Vec<TrackFuture> {

        let borrowed_ip = self.to_ip.read().unwrap();
        let borrowed_tcp_listener = self.tcp_listener.read().unwrap();

        let ip = borrowed_ip.as_ref().unwrap();
        let port = self.to_port.read().unwrap().unwrap();

        *self.tcp_stream.write().unwrap() = Some(borrowed_tcp_listener.as_ref().unwrap().available_streams().read().unwrap().get(&(ip.to_string(), port)).unwrap().clone());

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future = Box::new(Box::pin(async move { auto_self.tcp_write().await }));

        vec![future]
    }
    
}
