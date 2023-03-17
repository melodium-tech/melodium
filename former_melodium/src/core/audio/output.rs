
use std::thread::*;
use std::sync::{Arc, Weak, RwLock, Barrier};
use crate::core::prelude::*;
use async_std::channel::*;
use async_std::task::sleep;
use cpal::traits::{HostTrait, DeviceTrait};
use cpal::SampleRate;

#[derive(Debug)]
pub struct AudioOutputModel {

    host: Weak<ModelHost>,

    stream_thread: RwLock<Option<JoinHandle<()>>>,
    stream_end_barrier: Arc<Barrier>,

    stream_send: Sender<f32>,
    stream_recv: Receiver<f32>,

    early_end: RwLock<bool>,
}

impl AudioOutputModel {

    pub fn new(host: Weak<ModelHost>) -> Arc<dyn HostedModel> {

        let (send, recv) = unbounded();

        Arc::new(Self {
            host,
            stream_thread: RwLock::new(None),
            stream_end_barrier: Arc::new(Barrier::new(2)),
            stream_send: send,
            stream_recv: recv,
            early_end: RwLock::new(true),
        })
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
}


impl HostedModel for AudioOutputModel {

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

        let host = self.host.upgrade().unwrap();

        let auto_self = Arc::clone(host.hosted()).downcast_arc::<AudioOutputModel>().unwrap();
        let future = Box::pin(async move { auto_self.wait_for_init().await });

        host.world().add_continuous_task(Box::new(future));
    }

    fn shutdown(&self) {
        self.stream_recv.close();
        self.stream_end_barrier.wait();
    }
}

model!(
    AudioOutputModel,
    core_identifier!("audio";"AudioOutput"),
    indoc!(r"Represents host audio system output.

    It uses the default output device available.
    When `early_end` is set to `true`, the output is closed as soon as processing is done, _without taking care all sound has been played_, when to `false`, the output waits for all sound streams to be send to the system before exiting.
    This parameter should be turned to `false` especially when playing audio files that are fast to read and process, while they may contain long audio duration.").to_string(),
    parameters![
        parameter!("early_end", Scalar, Bool, Some(Value::Bool(true)))
    ],
    model_sources![
        ("send"; )
    ]
);

treatment!(send_audio_treatment,
    core_identifier!("audio";"SendAudio"),
    indoc!(r"Send audio to host system.

    Take a stream containing audio signal and send it to the host audio system.
    
    ⚠️ The signal must contain values between -1 and 1 (continuous 0 being silence).
    Other values will be handled by the host system in undefined way.").to_string(),
    models![
        ("output", crate::core::audio::output::model_host::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::output::model_host::descriptor(), "send")
    ],
    parameters![],
    inputs![
        input!("signal",Scalar,F32,Stream)
    ],
    outputs![],
    host {
        let input = host.get_input("signal");
        let audio_model = host.get_hosted_model("output").downcast_arc::<crate::core::audio::output::AudioOutputModel>().unwrap();
    
        'main: while let Ok(signal) = input.recv_f32().await {

            for sample in signal {
                ok_or_break!('main, audio_model.stream_send.send(sample).await);
            }
            
        }

        audio_model.stream_send.close();
    
        ResultStatus::Ok
    }
);
