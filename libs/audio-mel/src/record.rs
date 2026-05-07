use crate::audio_info::*;
use async_channel::bounded;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SampleFormat, SizedSample,
};
use melodium_core::*;
use melodium_macro::mel_treatment;
use std::sync::Arc;

/// Record from the default audio input device as a normalised mono `f32` signal.
///
/// Capture starts immediately when the treatment is activated. Samples are emitted through
/// `signal` as a continuous stream of `f32` values in the range `[-1.0, 1.0]`.
///
/// If the input device has more than one channel, all channels are mixed down to mono by
/// averaging each frame across channels before emission.
///
/// The sample format and channel count are determined automatically from the default
/// configuration reported by the host for the default input device. The optional `device`
/// parameter can be set to the name of a specific input device to use instead of the
/// system default. When `none`, the system default input device is used.
///
/// `errors` emits a message for every problem encountered during capture:
/// - recoverable problems (a transient stream error reported by the driver) produce one
///   message and capture continues;
/// - fatal problems (no input device available, unsupported sample format, stream build
///   failure) produce one message, trigger `failed`, and close all outputs immediately.
///
/// `failed` triggers if and only if capture cannot continue at all, regardless of whether
/// any samples were already emitted on `signal` before the failure occurred.
///
/// ⚠️ The treatment runs until the downstream consumer closes `signal` or until a fatal
/// error occurs. It does not stop on its own.
///
/// ```mermaid
/// graph LR
///     T("recordMono()")
///     T -->|signal| S["−0.3 … 0.7"]
///     T -->|info| I["AudioInfo"]
///     T -->|errors| E["…"]
///     T -->|failed| F["⬛"]
///
///     style S fill:#ffff,stroke:#ffff
///     style I fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input trigger Block<void>,
    output signal Stream<f32>
    output info Block<AudioInfo>
    output failed Block<void>
    output errors Stream<string>
)]
pub async fn record_mono(device: Option<string>) {
    if let Ok(_) = trigger.recv_one().await {
        // Trigger received, start capture. We ignore the value since it's just a signal.
        // Channel carrying interleaved f32 batches from the cpal callback to the async side.
        let (mono_sender, mono_receiver) = bounded::<Vec<f32>>(256);
        // Channel carrying error messages: (fatal, message).
        let (err_sender, err_receiver) = bounded::<(bool, String)>(64);
        // Channel carrying AudioInfo — sent once after the stream starts successfully.
        let (info_sender, info_receiver) = bounded::<AudioInfo>(1);

        // Future A: set up cpal and run the stream on the audio thread.
        // cpal stream callbacks are synchronous and run on a dedicated audio thread.
        // We keep the Stream alive inside spawn_blocking until mono_sender is disconnected
        // (i.e. the consumer stopped reading) or a fatal error occurs.
        let capture_fut = async move {
            let _ = async_std::task::spawn_blocking(move || {
                let host = cpal::default_host();

                let input_device = match &device {
                    None => match host.default_input_device() {
                        Some(d) => d,
                        None => {
                            let _ = async_std::task::block_on(
                                err_sender
                                    .send((true, "no default input device available".to_string())),
                            );
                            return;
                        }
                    },
                    Some(name) => {
                        let found = host.input_devices().ok().and_then(|mut iter| {
                            iter.find(|d| {
                                d.description()
                                    .map(|desc| desc.name() == name.as_str())
                                    .unwrap_or(false)
                            })
                        });
                        match found {
                            Some(d) => d,
                            None => {
                                let _ = async_std::task::block_on(
                                    err_sender
                                        .send((true, format!("input device not found: {name}"))),
                                );
                                return;
                            }
                        }
                    }
                };

                let config = match input_device.default_input_config() {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = async_std::task::block_on(
                            err_sender.send((true, format!("failed to get input config: {e}"))),
                        );
                        return;
                    }
                };

                let num_channels = config.channels() as usize;
                let stream_config = config.config();
                let sample_format = config.sample_format();

                let stream_result = build_stream(
                    &input_device,
                    &stream_config,
                    sample_format,
                    num_channels,
                    mono_sender,
                    err_sender.clone(),
                );

                let stream = match stream_result {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = async_std::task::block_on(
                            err_sender.send((true, format!("failed to build input stream: {e}"))),
                        );
                        return;
                    }
                };

                if let Err(e) = stream.play() {
                    let _ = async_std::task::block_on(
                        err_sender.send((true, format!("failed to start stream: {e}"))),
                    );
                    return;
                }

                let audio_info = AudioInfo {
                    codec: format!("{:?}", sample_format),
                    channels: num_channels as u32,
                    sample_rate: stream_config.sample_rate,
                    duration_seconds: None,
                };
                let _ = async_std::task::block_on(info_sender.send(audio_info));

                // Park this thread. The stream keeps running via its internal audio thread.
                // We unpark when mono_sender's receiver side is gone (consumer closed signal).
                let thread = std::thread::current();
                // Wait until the stream ends naturally (sender dropped by callback) or is stopped.
                // The callback drops mono_sender when the receiver is gone, causing send to fail,
                // but we need a signal back here. Use a simple loop with park_timeout as a heartbeat.
                loop {
                    std::thread::park_timeout(std::time::Duration::from_millis(100));
                    // If err_sender is disconnected (err_receiver dropped), stop.
                    if err_sender.is_closed() {
                        break;
                    }
                    // stream is kept alive by this scope; drop it when we exit.
                    let _ = &stream;
                    let _ = &thread;
                }
            })
            .await;
        };

        // Future B: forward captured mono chunks to the Mélodium signal output.
        let forward_fut = async {
            while let Ok(mono) = mono_receiver.recv().await {
                let batch: VecDeque<f32> = mono.into_iter().collect();
                if signal.send_many(batch.into()).await.is_err() {
                    break;
                }
            }
        };

        // Future C: receive AudioInfo and emit it on the `info` block output.
        let info_fut = async {
            if let Ok(audio_info) = info_receiver.recv().await {
                let _ = info.send_one(Value::Data(Arc::new(audio_info))).await;
            }
        };

        // Future D: forward error messages; trigger `failed` on the first fatal one.
        let error_fut = async {
            while let Ok((fatal, msg)) = err_receiver.recv().await {
                let _ = errors.send_one(msg.into()).await;
                if fatal {
                    let _ = failed.send_one(().into()).await;
                    break;
                }
            }
        };

        futures::join!(capture_fut, forward_fut, info_fut, error_fut);
    }
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    num_channels: usize,
    mono_sender: async_channel::Sender<Vec<f32>>,
    err_sender: async_channel::Sender<(bool, String)>,
) -> Result<cpal::Stream, String> {
    macro_rules! build {
        ($t:ty) => {{
            let sender = mono_sender.clone();
            let err = err_sender.clone();
            device
                .build_input_stream(
                    config,
                    move |data: &[$t], _: &cpal::InputCallbackInfo| {
                        let mono = mix_to_mono::<$t>(data, num_channels);
                        if async_std::task::block_on(sender.send(mono)).is_err() {
                            // receiver gone — stream will be dropped when capture_fut exits
                        }
                    },
                    move |e| {
                        let _ = async_std::task::block_on(
                            err.send((false, format!("stream error: {e}"))),
                        );
                    },
                    None,
                )
                .map_err(|e| e.to_string())
        }};
    }

    match sample_format {
        SampleFormat::F32 => build!(f32),
        SampleFormat::F64 => build!(f64),
        SampleFormat::I8 => build!(i8),
        SampleFormat::I16 => build!(i16),
        SampleFormat::I32 => build!(i32),
        SampleFormat::I64 => build!(i64),
        SampleFormat::U8 => build!(u8),
        SampleFormat::U16 => build!(u16),
        SampleFormat::U32 => build!(u32),
        SampleFormat::U64 => build!(u64),
        f => Err(format!("unsupported sample format: {f:?}")),
    }
}

fn mix_to_mono<T>(data: &[T], num_channels: usize) -> Vec<f32>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    if num_channels <= 1 {
        return data.iter().map(|&s| f32::from_sample_(s)).collect();
    }
    let inv = 1.0_f32 / num_channels as f32;
    data.chunks_exact(num_channels)
        .map(|frame| frame.iter().map(|&s| f32::from_sample_(s)).sum::<f32>() * inv)
        .collect()
}
