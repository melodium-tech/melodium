
use std::thread::*;
use std::sync::{Arc, Barrier};
use crate::core::prelude::*;
use async_std::channel::*;
use async_std::task::sleep;
use cpal::traits::{HostTrait, DeviceTrait};
use cpal::SampleRate;

#[derive(Debug)]
pub struct AudioOutputModel {

    helper: ModelHelper,

    stream_thread: RwLock<Option<JoinHandle<()>>>,
    stream_end_barrier: Arc<Barrier>,

    stream_send: Sender<f32>,
    stream_recv: Receiver<f32>,

    early_end: RwLock<bool>,

    auto_reference: Weak<Self>,
}

impl AudioOutputModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {

        model_desc!(
            AudioOutputModel,
            core_identifier!("audio";"AudioOutput"),
            vec![
                parameter!("early_end", Scalar, Bool, Some(Value::Bool(true))),
            ],
            model_sources![
                ("send"; )
            ]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        let (send, recv) = unbounded();

        Arc::new_cyclic(|me| Self {
            helper: ModelHelper::new(Self::descriptor(), world),

            stream_thread: RwLock::new(None),
            stream_end_barrier: Arc::new(Barrier::new(2)),

            stream_send: send,
            stream_recv: recv,

            early_end: RwLock::new(true),

            auto_reference: me.clone(),
        })
    }

    fn spawn_thread(&self) {

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

        let auto_self = self.auto_reference.upgrade().unwrap();
        let future = Box::pin(async move { auto_self.wait_for_init().await });

        self.helper.world().add_continuous_task(Box::new(future));
    }

    async fn wait_for_init(&self) {

        // Let time for the output thread to init with system audio service
        sleep(std::time::Duration::from_secs(1)).await;

        let early_end = *self.early_end.read().unwrap();

        if !early_end {
            while !self.stream_recv.is_empty() {
                sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }

    fn close_wait(&self) {

        self.stream_recv.close();
        self.stream_end_barrier.wait();
        //self.stream_thread.into_inner().unwrap().unwrap().join();
    }
}

model_trait!(AudioOutputModel, spawn_thread, close_wait);

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
