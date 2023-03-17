
use std::thread::*;
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock, Barrier};
use crate::core::prelude::*;
use async_std::channel::*;
use async_std::task::sleep;
use cpal::traits::{HostTrait, DeviceTrait};
use cpal::SampleRate;

#[derive(Debug)]
pub struct AudioInputModel {

    host: Weak<ModelHost>,

    stream_thread: RwLock<Option<JoinHandle<()>>>,
    stream_end_barrier: Arc<Barrier>,

    stream_send: Sender<Vec<f32>>,
    stream_recv: Receiver<Vec<f32>>,
}

impl AudioInputModel {

    pub fn new(host: Weak<ModelHost>) -> Arc<dyn HostedModel> {

        let (send, recv) = unbounded();

        Arc::new(Self {
            host,
            stream_thread: RwLock::new(None),
            stream_end_barrier: Arc::new(Barrier::new(2)),
            stream_send: send,
            stream_recv: recv,
        })
    }

    async fn receive(&self) {

        let host = self.host.upgrade().unwrap();

        let model_id = host.id().unwrap();

        sleep(std::time::Duration::from_secs(1)).await;

        let /*mut*/ contextes = HashMap::new();

        let mut recv = self.stream_recv.clone();
        let receiver = move |inputs: HashMap<String, Output>| {
            
            let future = Box::new(Box::pin(async move {

                let data_output = inputs.get("signal").unwrap();
    
                while let Some(possible_f32) = recv.next().await {
    
                    ok_or_break!(data_output.send_multiple_f32(possible_f32).await);
                }
    
                data_output.close().await;
    
                ResultStatus::Ok
            })) as TrackFuture;
    
            vec![future]
        };

        host.world().create_track(model_id, "receive", contextes, None, Some(receiver)).await;
    }
}

impl HostedModel for AudioInputModel {

    fn initialize(&self) {
        let sender = self.stream_send.clone();
        let barrier = Arc::clone(&self.stream_end_barrier);
        let stream_thread = spawn(move || {

            let host = cpal::default_host();

            if let Some(input_device) = host.default_input_device() {

                if let Ok(mut supported_config_range) = input_device.supported_input_configs() {

                    if let Some(supported_config) = supported_config_range.next() {

                        let config = supported_config.with_sample_rate(SampleRate(44100)).config();

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

        let host = self.host.upgrade().unwrap();

        let auto_self = Arc::clone(host.hosted()).downcast_arc::<AudioInputModel>().unwrap();
        let future = Box::pin(async move { auto_self.receive().await });

        host.world().add_continuous_task(Box::new(future));
    }

    fn shutdown(&self) {
        self.stream_recv.close();
        self.stream_end_barrier.wait();
    }
}

model!(
    AudioInputModel,
    core_identifier!("audio";"AudioInput"),
    indoc!(r"Represents host audio system input.

    It uses the default input device available.").to_string(),
    parameters![],
    model_sources![
        ("receive"; )
    ]
);

source!(receive_audio_source,
    core_identifier!("audio";"ReceiveAudio"),
    indoc!(r"Receive audio from host system.

    Outputs a stream containing audio signal.
    This stream contains `f32` values with amplitude being -1 and 1, and continuous 0 being silence.").to_string(),
    models![
        ("input", crate::core::audio::input::model_host::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::input::model_host::descriptor(), "receive")
    ],
    outputs![
        output!("signal",Scalar,F32,Stream)
    ]
);
