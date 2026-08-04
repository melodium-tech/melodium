use crate::audio_info::*;
use melodium_core::*;
use melodium_macro::mel_treatment;
use std::sync::Arc;

/// Resample a normalised mono `f32` signal from one sample rate to another.
///
/// Samples arrive through `signal` and are emitted through `resampled` at `to_rate` samples
/// per second. The source rate is read from `info`, which is the `AudioInfo` block produced
/// by `decodeMono` or `recordMono`. The amplitude range `[-1.0, 1.0]` is preserved.
///
/// When the source rate already equals `to_rate` the samples are forwarded unchanged.
///
/// Resampling is performed with linear interpolation. This is suitable for voice processing,
/// ML feature extraction, and similar tasks. For applications that require higher spectral
/// fidelity (e.g. professional audio mastering) a sinc-interpolating resampler such as
/// `rubato` would be preferable.
///
/// `resampled` closes as soon as `signal` closes, after all pending output samples have been
/// emitted. If `info` never arrives (e.g. decoding failed before any audio was probed)
/// `resampled` closes immediately without emitting any samples.
///
/// ```mermaid
/// graph LR
///     T("resampleMono()")
///     S["−0.3 … 0.7"] -->|signal| T
///     I["AudioInfo"] -->|info| T
///     T -->|resampled| R["−0.3 … 0.7 @ 16000 Hz"]
///
///     style S fill:#ffffff,stroke:#ffffff
///     style I fill:#ffffff,stroke:#ffffff
///     style R fill:#ffffff,stroke:#ffffff
/// ```
///
/// ```mel
/// use audio/decode::decodeMono
/// use audio/transform::resampleMono
/// use fs/local::readLocal
/// use std/engine/util::startup
///
/// treatment toMl()
///   output signal: Stream<f32>
/// {
///     startup()
///     readLocal(path="speech.wav")
///     decodeMono(hint="wav")
///     resampleMono(to_rate=16000)
///
///     startup.trigger -> readLocal.trigger,data -> decodeMono.data
///     decodeMono.signal -> resampleMono.signal,resampled -> Self.signal
///     decodeMono.info --> resampleMono.info
/// }
/// ```
#[mel_treatment(
    input  info      Block<AudioInfo>
    input  signal    Stream<f32>
    output resampled Stream<f32>
)]
pub async fn resample_mono(to_rate: u32) {
    // Wait for AudioInfo to learn the source sample rate before consuming any signal samples.
    let from_rate = match info.recv_one().await {
        Ok(value) => {
            let arc = GetData::<Arc<dyn Data>>::try_data(value).unwrap();
            let audio_info = arc.downcast_arc::<AudioInfo>().unwrap();
            audio_info.sample_rate
        }
        Err(_) => return,
    };

    if from_rate == to_rate || from_rate == 0 {
        // Passthrough: forward every batch unchanged.
        while let Ok(batch) = signal
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
        {
            let out: VecDeque<f32> = batch.into_iter().collect();
            if resampled.send_many(out.into()).await.is_err() {
                return;
            }
        }
        return;
    }

    // Ratio: how many input samples advance per output sample.
    let ratio = from_rate as f64 / to_rate as f64;

    // Buffer of input samples. We keep at least one sample from the previous batch so that
    // linear interpolation across batch boundaries works correctly.
    let mut buf: Vec<f32> = Vec::new();
    // Fractional read position inside `buf`, in input-sample units.
    let mut phase: f64 = 0.0;

    while let Ok(batch) = signal
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        buf.extend_from_slice(&batch);

        let mut out: VecDeque<f32> = VecDeque::new();

        while phase + 1.0 <= (buf.len() - 1) as f64 + 1.0 {
            let i = phase as usize;
            if i + 1 >= buf.len() {
                break;
            }
            let frac = (phase - i as f64) as f32;
            out.push_back(buf[i] + frac * (buf[i + 1] - buf[i]));
            phase += ratio;
        }

        if !out.is_empty() && resampled.send_many(out.into()).await.is_err() {
            return;
        }

        // Discard consumed samples, keeping the last as the left-hand interpolation point.
        let consumed = (phase as usize).saturating_sub(1);
        if consumed > 0 && consumed < buf.len() {
            buf.drain(..consumed);
            phase -= consumed as f64;
        }
    }

    // Flush: emit remaining output samples, holding the last input value for extrapolation.
    if !buf.is_empty() {
        let last = *buf.last().unwrap();
        let mut out: VecDeque<f32> = VecDeque::new();
        while phase < buf.len() as f64 {
            let i = phase as usize;
            let sample = if i + 1 < buf.len() {
                let frac = (phase - i as f64) as f32;
                buf[i] + frac * (buf[i + 1] - buf[i])
            } else {
                last
            };
            out.push_back(sample);
            phase += ratio;
        }
        if !out.is_empty() {
            let _ = resampled.send_many(out.into()).await;
        }
    }
}
