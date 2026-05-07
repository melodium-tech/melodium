use async_channel::bounded;
use flacenc::error::Verify;
use melodium_core::*;
use melodium_macro::mel_treatment;

/// Encode a normalised mono `f32` signal into a WAV byte stream.
///
/// Samples arrive through `signal` as `f32` values in the range `[-1.0, 1.0]`. They are
/// encoded as 32-bit IEEE float PCM and emitted through `data` as a continuous stream of
/// raw WAV bytes, suitable for writing directly to a file or sending over a network.
///
/// `sample_rate` must match the rate at which the samples in `signal` were produced.
///
/// `errors` emits a message if encoding fails. `failed` triggers on any fatal error.
///
/// ```mermaid
/// graph LR
///     T("encodeMonoWav()")
///     S["−0.3 … 0.7"] -->|signal| T
///     T -->|data| D["🟦 … 🟥"]
///     T -->|errors| E["…"]
///     T -->|failed| F["⬛"]
///
///     style S fill:#ffff,stroke:#ffff
///     style D fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input signal Stream<f32>
    output data Stream<byte>
    output failed Block<void>
    output errors Stream<string>
)]
pub async fn encode_mono_wav(sample_rate: u32) {
    // Collect all input samples first: hound needs to write the RIFF header up front
    // (which includes the total data length), so we buffer everything then encode in one pass.
    let (samples_sender, samples_receiver) = bounded::<Vec<f32>>(256);
    let (result_sender, result_receiver) = bounded::<Result<Vec<u8>, String>>(1);

    // Future A: drain signal into samples_sender.
    let collect_fut = async move {
        while let Ok(batch) = signal
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
        {
            if samples_sender.send(batch).await.is_err() {
                break;
            }
        }
    };

    // Future B: receive all samples and encode on a blocking thread.
    let encode_fut = async move {
        let _ = async_std::task::spawn_blocking(move || {
            let mut all_samples: Vec<f32> = Vec::new();
            while let Ok(batch) = samples_receiver.recv_blocking() {
                all_samples.extend_from_slice(&batch);
            }

            let spec = hound::WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            };

            let mut buf = std::io::Cursor::new(Vec::<u8>::new());
            let encode_result = (|| -> Result<Vec<u8>, String> {
                let mut writer = hound::WavWriter::new(&mut buf, spec)
                    .map_err(|e| format!("WAV writer init failed: {e}"))?;
                for &s in &all_samples {
                    writer
                        .write_sample(s)
                        .map_err(|e| format!("WAV write failed: {e}"))?;
                }
                writer
                    .finalize()
                    .map_err(|e| format!("WAV finalize failed: {e}"))?;
                Ok(buf.into_inner())
            })();

            let _ = async_std::task::block_on(result_sender.send(encode_result));
        })
        .await;
    };

    // Forward result to outputs.
    let forward_fut = async {
        if let Ok(result) = result_receiver.recv().await {
            match result {
                Ok(bytes) => {
                    let batch: VecDeque<u8> = bytes.into_iter().collect();
                    let _ = data.send_many(batch.into()).await;
                }
                Err(msg) => {
                    let _ = errors.send_one(msg.into()).await;
                    let _ = failed.send_one(().into()).await;
                }
            }
        }
    };

    futures::join!(collect_fut, encode_fut, forward_fut);
}

/// Encode a normalised mono `f32` signal into a FLAC byte stream.
///
/// Samples arrive through `signal` as `f32` values in the range `[-1.0, 1.0]`. They are
/// quantised to 24-bit signed integer PCM (preserving full audible dynamic range) and
/// encoded with the FLAC lossless codec. The resulting bytes are emitted through `data`.
///
/// `sample_rate` must match the rate at which the samples in `signal` were produced.
///
/// Because FLAC requires the full sample buffer before the stream header can be written,
/// all samples are buffered in memory before any bytes are emitted on `data`.
///
/// `errors` emits a message if encoding fails. `failed` triggers on any fatal error.
///
/// ```mermaid
/// graph LR
///     T("encodeMonoFlac()")
///     S["−0.3 … 0.7"] -->|signal| T
///     T -->|data| D["🟦 … 🟥"]
///     T -->|errors| E["…"]
///     T -->|failed| F["⬛"]
///
///     style S fill:#ffff,stroke:#ffff
///     style D fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input signal Stream<f32>
    output data Stream<byte>
    output failed Block<void>
    output errors Stream<string>
)]
pub async fn encode_mono_flac(sample_rate: u32) {
    let (samples_sender, samples_receiver) = bounded::<Vec<f32>>(256);
    let (result_sender, result_receiver) = bounded::<Result<Vec<u8>, String>>(1);

    let collect_fut = async move {
        while let Ok(batch) = signal
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
        {
            if samples_sender.send(batch).await.is_err() {
                break;
            }
        }
    };

    let encode_fut = async move {
        let _ = async_std::task::spawn_blocking(move || {
            let mut all_samples: Vec<f32> = Vec::new();
            while let Ok(batch) = samples_receiver.recv_blocking() {
                all_samples.extend_from_slice(&batch);
            }

            let encode_result = (|| -> Result<Vec<u8>, String> {
                // Quantise f32 [-1.0, 1.0] → i32 with 24-bit range.
                const SCALE: f32 = (1i32 << 23) as f32;
                let pcm: Vec<i32> = all_samples
                    .iter()
                    .map(|&s| (s.clamp(-1.0, 1.0) * SCALE) as i32)
                    .collect();

                let source =
                    flacenc::source::MemSource::from_samples(&pcm, 1, 24, sample_rate as usize);

                let config = flacenc::config::Encoder::default()
                    .into_verified()
                    .map_err(|(_, e)| format!("FLAC config error: {e}"))?;

                let flac_stream = flacenc::encode_with_fixed_block_size(&config, source, 4096)
                    .map_err(|e| format!("FLAC encode failed: {e}"))?;

                let mut sink = flacenc::bitsink::ByteSink::new();
                flacenc::component::BitRepr::write(&flac_stream, &mut sink)
                    .map_err(|e| format!("FLAC write failed: {e}"))?;

                Ok(sink.as_slice().to_vec())
            })();

            let _ = async_std::task::block_on(result_sender.send(encode_result));
        })
        .await;
    };

    let forward_fut = async {
        if let Ok(result) = result_receiver.recv().await {
            match result {
                Ok(bytes) => {
                    let batch: VecDeque<u8> = bytes.into_iter().collect();
                    let _ = data.send_many(batch.into()).await;
                }
                Err(msg) => {
                    let _ = errors.send_one(msg.into()).await;
                    let _ = failed.send_one(().into()).await;
                }
            }
        }
    };

    futures::join!(collect_fut, encode_fut, forward_fut);
}
