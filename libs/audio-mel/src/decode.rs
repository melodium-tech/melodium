use async_channel::bounded;
use melodium_core::*;
use melodium_macro::mel_treatment;
use std::sync::Arc;
use symphonia::core::{
    audio::AudioBuffer,
    codecs::DecoderOptions,
    errors::Error,
    formats::FormatOptions,
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    probe::Hint,
};
use crate::audio_info::*;
use crate::channels::*;

/// Decode an audio stream of any supported format into a normalised mono `f32` signal.
///
/// Audio bytes arrive through `data` and are decoded on-the-fly as they stream in, without
/// buffering the entire input first. The decoded samples are emitted through `signal` as a
/// continuous stream of `f32` values in the range `[-1.0, 1.0]`, regardless of the bit depth
/// or sample format of the source.
///
/// If the source audio has more than one channel, all channels are mixed down to mono by
/// averaging each frame across channels before emission.
///
/// The format is detected automatically from the content of the stream. The optional `hint`
/// parameter can be set to a file extension (e.g. `"mp3"`, `"flac"`, `"ogg"`, `"wav"`) to
/// help the detection when the content alone is ambiguous. When `none`, detection relies
/// entirely on the byte content.
///
/// Supported formats and codecs:
/// - **WAVE** (`.wav`): PCM (all standard bit depths: 8, 16, 24, 32-bit integer and
///   32/64-bit float), ADPCM (Microsoft and IMA/DVI variants), A-law, μ-law.
/// - **AIFF** (`.aiff`, `.aif`): PCM.
/// - **FLAC** (`.flac`): Free Lossless Audio Codec, all standard bit depths.
/// - **Ogg** (`.ogg`, `.oga`): Vorbis audio inside an Ogg container.
/// - **MP3** (`.mp3`): MPEG-1 Audio Layer I, II, and III.
/// - **Matroska / WebM** (`.mkv`, `.mka`, `.webm`): any audio track whose codec is
///   otherwise supported (e.g. PCM, FLAC, Vorbis, MP3).
/// - **ISO Base Media / MP4 / M4A** (`.mp4`, `.m4a`, `.aac`): AAC, ALAC, and other
///   codecs carried in an ISOBMFF container.
/// - **Core Audio Format** (`.caf`): PCM and other codecs in Apple's CAF container.
///
/// `errors` emits a message for every problem encountered during decoding:
/// - recoverable problems (a single corrupt packet that is skipped) produce one message and
///   decoding continues;
/// - fatal problems (unrecognised format, missing audio track, unsupported codec, unrecoverable
///   I/O error) produce one message, trigger `failed`, and close all outputs immediately.
///
/// `failed` triggers if and only if decoding cannot continue at all, regardless of whether any
/// samples were already emitted on `signal` before the failure occurred.
///
/// ⚠️ `data` must carry a single, complete audio stream from start to finish. Mixing bytes
/// from different files or restarting mid-stream will cause detection or decoding to fail.
///
/// ```mermaid
/// graph LR
///     T("decodeMono()")
///     D["🟦 … 🟥"] -->|data| T
///     T -->|signal| S["−0.3 … 0.7"]
///     T -->|info| I["AudioInfo"]
///     T -->|errors| E["…"]
///     T -->|failed| F["⬛"]
///
///     style D fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
///     style I fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
/// ```
///
/// ```
/// use audio/decode::decodeMono
/// use fs/local::readLocal
/// use std/engine/util::startup
///
/// treatment playFlacFile()
///   output signal: Stream<f32>
/// {
///     startup()
///     readLocal(path="track.flac")
///     decodeMono(hint="flac")
///
///     startup.trigger -> readLocal.trigger,data -> decodeMono.data,signal -> Self.signal
/// }
/// ```
#[mel_treatment(
    input data Stream<byte>
    output signal Stream<f32>
    output info Block<AudioInfo>
    output failed Block<void>
    output errors Stream<string>
)]
pub async fn decode_mono(hint: Option<string>) {
    // Channel carrying raw byte chunks from the feed loop to the symphonia reader.
    let (byte_sender, byte_receiver) = bounded::<Vec<u8>>(1024);
    // Channel carrying decoded mono f32 batches from the decode loop to the signal output loop.
    let (mono_sender, mono_receiver) = bounded::<Vec<f32>>(64);
    // Channel carrying error messages from the blocking thread back to the async side.
    // Each item is (fatal: bool, message: String). A fatal message is always the last one sent.
    let (err_sender, err_receiver) = bounded::<(bool, String)>(64);
    // Channel carrying AudioInfo from the blocking thread — sent exactly once after probing.
    let (info_sender, info_receiver) = bounded::<AudioInfo>(1);

    let mut symphonia_hint = Hint::new();
    if let Some(ref h) = hint {
        symphonia_hint.with_extension(h.as_str());
    }

    // Future A: probe → decode → send mono chunks into `mono_sender`.
    // Symphonia's format reader is synchronous, so all blocking work runs on a
    // dedicated thread via spawn_blocking. Results bridge back through mono_sender.
    let decode_fut = async move {
        let _ = async_std::task::spawn_blocking(move || {
            let _ = &info_sender; // ensure it is moved into the closure
            let mss = MediaSourceStream::new(
                Box::new(ChannelReaderMediaSource::new(byte_receiver)),
                MediaSourceStreamOptions::default(),
            );

            let probe = symphonia::default::get_probe();

            let probe_result = match probe.format(
                &symphonia_hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            ) {
                Ok(r) => r,
                Err(e) => {
                    let _ = async_std::task::block_on(
                        err_sender.send((true, format!("format detection failed: {e}")))
                    );
                    return;
                }
            };

            let mut format_reader = probe_result.format;

            let track = match format_reader.default_track() {
                Some(t) => t.clone(),
                None => {
                    let _ = async_std::task::block_on(
                        err_sender.send((true, "no audio track found".to_string()))
                    );
                    return;
                }
            };

            let codec_registry = symphonia::default::get_codecs();
            let mut decoder = match codec_registry
                .make(&track.codec_params, &DecoderOptions::default())
            {
                Ok(d) => d,
                Err(e) => {
                    let _ = async_std::task::block_on(
                        err_sender.send((true, format!("unsupported codec: {e}")))
                    );
                    return;
                }
            };

            let duration_seconds = track.codec_params.time_base
                .zip(track.codec_params.n_frames)
                .map(|(tb, n)| {
                    let t = tb.calc_time(n);
                    t.seconds as f64 + t.frac
                });
            let audio_info = AudioInfo {
                codec: format!("{}", track.codec_params.codec),
                channels: track.codec_params.channels
                    .map(|c| c.count() as u32)
                    .unwrap_or(0),
                sample_rate: track.codec_params.sample_rate.unwrap_or(0),
                duration_seconds,
            };
            let _ = async_std::task::block_on(info_sender.send(audio_info));

            let mut f32_buf: AudioBuffer<f32> = AudioBuffer::unused();

            loop {
                let packet = match format_reader.next_packet() {
                    Ok(p) => p,
                    Err(Error::IoError(e)) => {
                        let _ = async_std::task::block_on(
                            err_sender.send((true, format!("I/O error: {e}")))
                        );
                        break;
                    }
                    Err(Error::ResetRequired) => break,
                    Err(Error::DecodeError(e)) => {
                        let _ = async_std::task::block_on(
                            err_sender.send((false, format!("packet skipped: {e}")))
                        );
                        continue;
                    }
                    Err(e) => {
                        let _ = async_std::task::block_on(
                            err_sender.send((true, format!("demux error: {e}")))
                        );
                        break;
                    }
                };

                let audio_buf = match decoder.decode(&packet) {
                    Ok(buf) => buf,
                    Err(Error::DecodeError(e)) => {
                        let _ = async_std::task::block_on(
                            err_sender.send((false, format!("packet skipped: {e}")))
                        );
                        continue;
                    }
                    Err(e) => {
                        let _ = async_std::task::block_on(
                            err_sender.send((true, format!("decode error: {e}")))
                        );
                        break;
                    }
                };

                audio_buf.convert(&mut f32_buf);

                let planes = f32_buf.planes();
                let channel_slices = planes.planes();
                let num_channels = channel_slices.len();

                if num_channels == 0 {
                    continue;
                }

                let num_frames = channel_slices[0].len();

                let mono: Vec<f32> = if num_channels == 1 {
                    channel_slices[0].to_vec()
                } else {
                    let inv = 1.0_f32 / num_channels as f32;
                    (0..num_frames)
                        .map(|frame| {
                            channel_slices.iter().map(|ch| ch[frame]).sum::<f32>() * inv
                        })
                        .collect()
                };

                if async_std::task::block_on(mono_sender.send(mono)).is_err() {
                    break;
                }
            }
        })
        .await;
    };

    // Future B: receive data from Mélodium input and forward into byte_sender.
    let feed_fut = async move {
        while let Ok(chunk) = data
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<u8>>::try_into(values).unwrap())
        {
            if byte_sender.send(chunk).await.is_err() {
                break;
            }
        }
        // Dropping byte_sender signals EOF to the media source reader.
    };

    // Future C: forward decoded mono chunks to the Mélodium signal output.
    let forward_fut = async {
        while let Ok(mono) = mono_receiver.recv().await {
            let batch: VecDeque<f32> = mono.into_iter().collect();
            if signal.send_many(batch.into()).await.is_err() {
                break;
            }
        }
    };

    // Future D: receive AudioInfo and emit it on the `info` block output.
    let info_fut = async {
        if let Ok(audio_info) = info_receiver.recv().await {
            let _ = info.send_one(Value::Data(Arc::new(audio_info))).await;
        }
    };

    // Future E: forward error messages; trigger `failed` on the first fatal one.
    let error_fut = async {
        while let Ok((fatal, msg)) = err_receiver.recv().await {
            let _ = errors.send_one(msg.into()).await;
            if fatal {
                let _ = failed.send_one(().into()).await;
                break;
            }
        }
    };

    // `decode_fut` owns `byte_receiver` (via spawn_blocking), `mono_sender`, `err_sender`,
    // and `info_sender`; `feed_fut` owns `byte_sender`; `forward_fut` owns `mono_receiver`
    // and `signal`; `info_fut` owns `info_receiver` and `info`; `error_fut` owns the rest.
    futures::join!(decode_fut, feed_fut, forward_fut, info_fut, error_fut);
}
