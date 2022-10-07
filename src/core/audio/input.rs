
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

    helper: ModelHelper,

    stream_thread: RwLock<Option<JoinHandle<()>>>,
    stream_end_barrier: Arc<Barrier>,

    stream_send: Sender<Vec<f32>>,
    stream_recv: Receiver<Vec<f32>>,

    auto_reference: Weak<Self>,
}

impl AudioInputModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        model_desc!(
            AudioInputModel,
            core_identifier!("audio";"AudioInput"),
            vec![],
            model_sources![
                ("receive"; )
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

            auto_reference: me.clone(),
        })
    }

    fn spawn_thread(&self) {

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

        let auto_self = self.auto_reference.upgrade().unwrap();
        let future = Box::pin(async move { auto_self.receive().await });

        self.helper.world().add_continuous_task(Box::new(future));
    }

    async fn receive(&self) {

        let model_id = self.helper.id().unwrap();

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

        self.helper.world().create_track(model_id, "receive", contextes, None, Some(receiver)).await;
    }

    fn close_wait(&self) {

        self.stream_recv.close();
        self.stream_end_barrier.wait();
        //self.stream_thread.into_inner().unwrap().unwrap().join();
    }
}

model_trait!(AudioInputModel, spawn_thread, close_wait);

source!(receive_audio_source,
    core_identifier!("audio";"ReceiveAudio"),
    models![
        ("input", crate::core::audio::input::AudioInputModel::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::input::AudioInputModel::descriptor(), "receive")
    ],
    outputs![
        output!("signal",Scalar,F32,Stream)
    ]
);
