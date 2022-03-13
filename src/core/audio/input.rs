
use std::thread::*;
use std::sync::{Arc, Barrier};
use crate::core::prelude::*;
use async_std::channel::*;
use cpal::traits::{HostTrait, DeviceTrait};

#[derive(Debug)]
pub struct AudioInputModel {

    world: Arc<World>,
    id: RwLock<Option<ModelId>>,

    stream_thread: RwLock<Option<JoinHandle<()>>>,
    stream_end_barrier: Arc<Barrier>,

    stream_send: Sender<Vec<f32>>,
    stream_recv: Receiver<Vec<f32>>,

    auto_reference: RwLock<Weak<Self>>,
}

impl AudioInputModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {
                
                let builder = CoreModelBuilder::new(AudioInputModel::new);

                let descriptor = CoreModelDescriptor::new(
                    core_identifier!("audio";"AudioInput"),
                    vec![],
                    model_sources![
                        ("receive"; )
                    ],
                    Box::new(builder)
                );

                let rc_descriptor = Arc::new(descriptor);
                rc_descriptor.set_autoref(&rc_descriptor);

                rc_descriptor
            };
        }
        
        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        let (send, recv) = unbounded();

        let model = Arc::new(Self {
            world,
            id: RwLock::new(None),

            stream_thread: RwLock::new(None),
            stream_end_barrier: Arc::new(Barrier::new(2)),

            stream_send: send,
            stream_recv: recv,

            auto_reference: RwLock::new(Weak::new()),
        });

        *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

        model
    }

    async fn receive(&self) {

        let model_id = self.id.read().unwrap().unwrap();

        let /*mut*/ contextes = HashMap::new();

        let mut recv = self.stream_recv.clone();
        let receiver = move |inputs: HashMap<String, Vec<Input>>| {
            
            let future = Box::new(Box::pin(async move {

                let data_output = Output::F32(Arc::new(SendTransmitter::new()));
                inputs.get("_signal").unwrap().iter().for_each(|i| data_output.add_input(i));
    
                while let Some(possible_f32) = recv.next().await {
    
                    ok_or_break!(data_output.send_multiple_f32(possible_f32).await);
                }
    
                data_output.close().await;
    
                ResultStatus::Ok
            })) as TrackFuture;
    
            vec![future]
        };
        self.world.create_track(model_id, "receive", contextes, None, Some(receiver)).await;
    }
}

impl Model for AudioInputModel {
    
    fn descriptor(&self) -> Arc<CoreModelDescriptor> {
        Self::descriptor()
    }

    fn id(&self) -> Option<ModelId> {
        *self.id.read().unwrap()
    }

    fn set_id(&self, id: ModelId) {
        *self.id.write().unwrap() = Some(id);
    }

    fn set_parameter(&self, param: &str, value: &Value) {

        match param {
            _ => panic!("No parameter '{}' exists.", param)
        }
    }

    fn get_context_for(&self, source: &str) -> Vec<String> {

        match source {
            "receive" => vec![],
            _ => Vec::new(),
        }
    }

    fn initialize(&self) {

        let sender = self.stream_send.clone();
        let barrier = Arc::clone(&self.stream_end_barrier);
        let stream_thread = spawn(move || {
            let host = cpal::default_host();
            if let Some(input_device) = host.default_input_device() {

                if let Ok(mut supported_config_range) = input_device.supported_input_configs() {

                    if let Some(supported_config) = supported_config_range.next() {

                        let config = supported_config.with_max_sample_rate().config();

                        if let Ok(_stream) = input_device.build_input_stream(
                            &config,
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {

                                let vec = Vec::from(data);
                                let _ = async_std::task::block_on(async { sender.send(vec).await });
                            },
                            move |_err| {

                            }
                        ){

                            barrier.wait();
                        }
                    }
                }
            }
        });
        
        *self.stream_thread.write().unwrap() = Some(stream_thread);

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future = Box::pin(async move { auto_self.receive().await });

        self.world.add_continuous_task(Box::new(future));
    }

    fn shutdown(&self) {

        self.stream_recv.close();
        self.stream_end_barrier.wait();
        //self.stream_thread.into_inner().unwrap().unwrap().join();
    }
}

treatment!(receive_audio_treatment,
    core_identifier!("audio";"ReceiveAudio"),
    models![
        ("input", crate::core::audio::input::AudioInputModel::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::input::AudioInputModel::descriptor(), "receive")
    ],
    parameters![],
    inputs![
        input!("_signal",Scalar,F32,Stream)
    ],
    outputs![
        output!("signal",Scalar,F32,Stream)
    ],
    host {
        let input = host.get_input("_signal");
        let output = host.get_output("signal");
    
        while let Ok(signal) = input.recv_f32().await {

            ok_or_break!(output.send_multiple_f32(signal).await);
        }
    
        ResultStatus::Ok
    }
);
