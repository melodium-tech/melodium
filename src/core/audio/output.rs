
use std::thread::*;
use std::sync::{Arc, Barrier};
use crate::core::prelude::*;
use async_std::channel::*;
use async_std::task::sleep;
use cpal::traits::{HostTrait, DeviceTrait};
use cpal::SampleRate;

#[derive(Debug)]
pub struct AudioOutputModel {

    world: Arc<World>,
    id: RwLock<Option<ModelId>>,

    stream_thread: RwLock<Option<JoinHandle<()>>>,
    stream_end_barrier: Arc<Barrier>,

    stream_send: Sender<f32>,
    stream_recv: Receiver<f32>,

    auto_reference: RwLock<Weak<Self>>,
}

impl AudioOutputModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {
                
                let builder = CoreModelBuilder::new(AudioOutputModel::new);

                let descriptor = CoreModelDescriptor::new(
                    core_identifier!("audio";"AudioOutput"),
                    vec![],
                    model_sources![
                        ("send"; )
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

    async fn wait_for_init(&self) {

        // Let time for the output thread to init with system audio service
        sleep(std::time::Duration::from_secs(1)).await;
    }
}

impl Model for AudioOutputModel {
    
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

        vec![]
    }

    fn initialize(&self) {

        let receiver = self.stream_recv.clone();
        let barrier = Arc::clone(&self.stream_end_barrier);
        let stream_thread = spawn(move || {

            let host = cpal::default_host();

            if let Some(output_device) = host.default_output_device() {

                if let Ok(mut supported_config_range) = output_device.supported_output_configs() {

                    if let Some(supported_config) = supported_config_range.next() {

                        let config = supported_config.with_sample_rate(SampleRate(44100)).config();

                        if let Ok(_stream) = output_device.build_output_stream(
                            &config,
                            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {

                                for sample in output.iter_mut() {
                                    if let Ok(input) = receiver.try_recv() {
                                        *sample = input
                                    }
                                    else {
                                        *sample = 0.0
                                    }
                                }

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
        let future = Box::pin(async move { auto_self.wait_for_init().await });

        self.world.add_continuous_task(Box::new(future));
    }

    fn shutdown(&self) {

        self.stream_recv.close();
        self.stream_end_barrier.wait();
        //self.stream_thread.into_inner().unwrap().unwrap().join();
    }
}

treatment!(send_audio_treatment,
    core_identifier!("audio";"SendAudio"),
    models![
        ("output", crate::core::audio::output::AudioOutputModel::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::output::AudioOutputModel::descriptor(), "send")
    ],
    parameters![],
    inputs![
        input!("signal",Scalar,F32,Stream)
    ],
    outputs![],
    host {
        let input = host.get_input("signal");
        let audio_model = Arc::clone(&host.get_model("output")).downcast_arc::<crate::core::audio::output::AudioOutputModel>().unwrap();
    
        'main: while let Ok(signal) = input.recv_f32().await {

            for sample in signal {
                ok_or_break!('main, audio_model.stream_send.send(sample).await);
            }
            
        }

        audio_model.stream_send.close();
    
        ResultStatus::Ok
    }
);
