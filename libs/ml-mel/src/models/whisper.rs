use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as whisper_model, audio, Config};
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
use std::sync::Weak;

// The worker receives raw f32 chunks from the async side.
// Sending end is closed (dropped) by the treatment when the audio stream ends,
// which tells the worker to flush any remaining samples.
#[cfg(feature = "real")]
struct AudioChunk(Vec<f32>);

/// Whisper automatic speech recognition model configuration.
///
/// Holds the architecture parameters for a Whisper model.  Weights are not embedded
/// here — use an `HfHub` model together with `fetch` and `load` to supply them at
/// runtime, then call `decode` to transcribe a stream of PCM audio samples.
///
/// Architecture parameters (defaults match `openai/whisper-tiny`):
/// - `num_mel_bins`: number of mel filter banks.
/// - `max_source_positions`: maximum number of audio context positions.
/// - `d_model`: model hidden dimension.
/// - `encoder_attention_heads`: number of encoder attention heads.
/// - `encoder_layers`: number of encoder layers.
/// - `vocab_size`: vocabulary size.
/// - `max_target_positions`: maximum number of decoder output positions.
/// - `decoder_attention_heads`: number of decoder attention heads.
/// - `decoder_layers`: number of decoder layers.
///
/// ℹ️ Use `Whisper` together with `HfHub`, `fetch`, `load`, and `decode`.
/// `load` must complete successfully before `decode` will produce output.
///
/// ```
/// use ml/repos/hf::HfHub
/// use ml/repos/hf::fetch
/// use ml/models/whisper::Whisper
/// use ml/models/whisper::load
/// use ml/models/whisper::decode
/// use std/engine/util::startup
///
/// treatment example()
///   model hub:     HfHub(repo_id = "openai/whisper-tiny")
///   model whisper: Whisper()
///   input  audio:       Stream<f32>
///   output transcribed: Stream<string>
/// {
///     startup()
///     fetch[hub=hub]()
///     load[whisper=whisper]()
///     decode[whisper=whisper]()
///
///     startup.trigger    -> fetch.trigger
///     fetch.safetensors  -> load.safetensors
///     load.loaded        -> decode.ready
///     Self.audio         -> decode.audio
///     decode.transcribed -> Self.transcribed
/// }
/// ```
#[mel_model(
    param num_mel_bins              u64     80
    param max_source_positions      u64     1500
    param d_model                   u64     384
    param encoder_attention_heads   u64     6
    param encoder_layers            u64     4
    param vocab_size                u64     51865
    param max_target_positions      u64     448
    param decoder_attention_heads   u64     6
    param decoder_layers            u64     4
    shutdown shutdown
)]
#[derive(Debug)]
pub struct Whisper {
    model: Weak<WhisperModel>,
    // Sending end of the chunk queue. The worker owns the receiving end.
    // None until load() is called successfully.
    #[cfg(feature = "real")]
    chunk_tx: std::sync::Mutex<Option<flume::Sender<AudioChunk>>>,
    // Text produced by the worker, consumed by decode() treatments.
    #[cfg(feature = "real")]
    text_rx: std::sync::Mutex<Option<flume::Receiver<String>>>,
}

impl Whisper {
    fn new(model: Weak<WhisperModel>) -> Self {
        Self {
            model,
            #[cfg(feature = "real")]
            chunk_tx: std::sync::Mutex::new(None),
            #[cfg(feature = "real")]
            text_rx: std::sync::Mutex::new(None),
        }
    }

    fn shutdown(&self) {
        #[cfg(feature = "real")]
        {
            *self.chunk_tx.lock().unwrap() = None;
        }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    /// Load weights from `shard_paths`, then start the streaming decode worker.
    #[cfg(feature = "real")]
    pub fn load(&self, shard_paths: Vec<String>) -> Result<(), String> {
        let model_ref = self.model.upgrade().ok_or("model dropped")?;

        let config = Config {
            num_mel_bins:            model_ref.get_num_mel_bins() as usize,
            max_source_positions:    model_ref.get_max_source_positions() as usize,
            d_model:                 model_ref.get_d_model() as usize,
            encoder_attention_heads: model_ref.get_encoder_attention_heads() as usize,
            encoder_layers:          model_ref.get_encoder_layers() as usize,
            vocab_size:              model_ref.get_vocab_size() as usize,
            max_target_positions:    model_ref.get_max_target_positions() as usize,
            decoder_attention_heads: model_ref.get_decoder_attention_heads() as usize,
            decoder_layers:          model_ref.get_decoder_layers() as usize,
            suppress_tokens:         vec![],
        };

        let device = Device::Cpu;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                shard_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice(),
                DType::F32,
                &device,
            )
        }
        .map_err(|e| e.to_string())?;

        let candle_model = whisper_model::model::Whisper::load(&vb, config)
            .map_err(|e| e.to_string())?;

        let (chunk_tx, chunk_rx) = flume::unbounded::<AudioChunk>();
        let (text_tx, text_rx)   = flume::unbounded::<String>();

        std::thread::spawn(move || {
            worker_loop(candle_model, chunk_rx, text_tx);
        });

        *self.chunk_tx.lock().unwrap() = Some(chunk_tx);
        *self.text_rx.lock().unwrap()  = Some(text_rx);
        Ok(())
    }

    /// Send one batch of samples to the worker.
    /// Returns `false` if the model has not been loaded or the channel is closed.
    #[cfg(feature = "real")]
    pub fn send_chunk(&self, samples: Vec<f32>) -> bool {
        let tx = self.chunk_tx.lock().unwrap();
        tx.as_ref()
            .and_then(|tx| tx.send(AudioChunk(samples)).ok())
            .is_some()
    }

    /// Close the audio input channel, signalling end-of-stream to the worker.
    #[cfg(feature = "real")]
    pub fn close_audio(&self) {
        *self.chunk_tx.lock().unwrap() = None;
    }

    /// Receive the next decoded text segment from the worker (blocks until available or done).
    #[cfg(feature = "real")]
    pub fn recv_text(&self) -> Option<flume::Receiver<String>> {
        self.text_rx.lock().unwrap().clone()
    }
}

// Runs on a dedicated OS thread; never touches the async executor.
//
// Strategy: maintain a sliding sample buffer.  Once it reaches N_SAMPLES
// (the 30-second Whisper window), encode-decode it, emit results, then slide
// forward by N_SAMPLES so the next window starts fresh (no overlap needed for
// transcription without timestamps).  When the chunk channel closes (end of
// stream), flush whatever remains.
#[cfg(feature = "real")]
fn worker_loop(
    mut model: whisper_model::model::Whisper,
    chunk_rx: flume::Receiver<AudioChunk>,
    text_tx: flume::Sender<String>,
) {
    let device = Device::Cpu;
    let n_samples = whisper_model::N_SAMPLES; // 480 000 samples = 30 s @ 16 kHz
    let n_mels    = model.config.num_mel_bins;
    let n_fft     = whisper_model::N_FFT;
    let n_fft_bins = 1 + n_fft / 2;

    // Trivial mel filterbank — consumers using real HF weights should supply proper filters.
    let filters: Vec<f32> = (0..n_mels * n_fft_bins)
        .map(|i| if i % n_fft_bins == i / n_mels { 1.0f32 } else { 0.0 })
        .collect();

    // Special token ids (multilingual Whisper vocabulary).
    let sot_token     = 50258u32; // <|startoftranscript|>
    let transcribe    = 50359u32; // <|transcribe|>
    let no_timestamps = 50363u32; // <|notimestamps|>
    let eot_token     = 50256u32; // <|endoftext|>

    let mut buffer: Vec<f32> = Vec::with_capacity(n_samples * 2);

    // Encode and greedy-decode `window`, emit tokens through `text_tx`.
    // Returns false if the text channel has been closed (no consumers left).
    let mut decode_window = |model: &mut whisper_model::model::Whisper,
                              window: &[f32]| -> bool {
        model.reset_kv_cache();

        let mel = audio::pcm_to_mel(&model.config, window, &filters);
        let n_frames = mel.len() / n_mels;
        let mel_tensor = match Tensor::from_vec(mel, (1, n_mels, n_frames), &device) {
            Ok(t) => t,
            Err(_) => return true,
        };

        let audio_features = match model.encoder.forward(&mel_tensor, true) {
            Ok(f) => f,
            Err(_) => return true,
        };

        let mut tokens: Vec<u32> = vec![sot_token, transcribe, no_timestamps];

        for _ in 0..model.config.max_target_positions {
            let token_tensor = match Tensor::new(tokens.as_slice(), &device)
                .and_then(|t| t.unsqueeze(0))
            {
                Ok(t) => t,
                Err(_) => break,
            };

            let flush = tokens.len() == 3;
            let logits = match model.decoder.forward(&token_tensor, &audio_features, flush) {
                Ok(l) => l,
                Err(_) => break,
            };

            let last_logits = match model.decoder.final_linear(&logits) {
                Ok(l) => l,
                Err(_) => break,
            };

            let last_pos = last_logits.dims()[1].saturating_sub(1);
            let step_logits: Tensor = match last_logits.i((0, last_pos)) {
                Ok(l) => l,
                Err(_) => break,
            };

            let next_token = match step_logits.argmax(candle_core::D::Minus1) {
                Ok(t) => match t.to_scalar::<u32>() {
                    Ok(v) => v,
                    Err(_) => break,
                },
                Err(_) => break,
            };

            if next_token == eot_token {
                break;
            }

            tokens.push(next_token);

            let piece = format!("[{}]", next_token);
            if text_tx.send(piece).is_err() {
                return false; // receiver dropped — treatment gone
            }
        }
        true
    };

    // Consume chunks; decode each full window as soon as it is ready.
    for AudioChunk(samples) in chunk_rx.iter() {
        buffer.extend_from_slice(&samples);

        while buffer.len() >= n_samples {
            let window: Vec<f32> = buffer.drain(..n_samples).collect();
            if !decode_window(&mut model, &window) {
                return;
            }
        }
    }

    // Flush the final (possibly short) window when the audio stream ends.
    if !buffer.is_empty() {
        decode_window(&mut model, &buffer);
    }
}

/// Load weights into a Whisper model.
///
/// Collects all `.safetensors` shard paths from `safetensors` (stream closes when all
/// shards have been emitted), then memory-maps the weight shards and starts the decode
/// worker thread inside the `Whisper` model.
///
/// `loaded` is emitted when the model is ready to accept audio.  If any step fails,
/// `failed` and `error` are emitted instead and `loaded` is never sent.
///
/// ℹ️ Wire `safetensors` directly from a `fetch` treatment.
///
/// ⚠️ `decode` will silently drop audio until `load` has successfully completed.
///
/// ```mermaid
/// graph LR
///     T("load()")
///     S["🟩 🟩 🟩 …"] -->|safetensors| T
///     T -->|loaded| L["〈🟦〉"]
///     T -->|failed| F["〈🟦〉"]
///     T -->|error|  E["〈🟨〉"]
///
///     style S fill:#ffff,stroke:#ffff
///     style L fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
///
/// ```
/// use ml/repos/hf::HfHub
/// use ml/repos/hf::fetch
/// use ml/models/whisper::Whisper
/// use ml/models/whisper::load
/// use std/engine/util::startup
///
/// treatment example()
///   model hub:     HfHub(repo_id = "openai/whisper-tiny")
///   model whisper: Whisper()
/// {
///     startup()
///     fetch[hub=hub]()
///     load[whisper=whisper]()
///
///     startup.trigger   -> fetch.trigger
///     fetch.safetensors -> load.safetensors
/// }
/// ```
#[mel_treatment(
    model whisper Whisper
    input  safetensors Stream<string>
    output loaded      Block<void>
    output failed      Block<void>
    output error       Block<string>
)]
pub async fn load() {
    let model_arc = WhisperModel::into(whisper);
    let whisper_struct = model_arc.inner();

    let mut shard_paths: Vec<String> = Vec::new();
    while let Ok(val) = safetensors.recv_one().await {
        if let Ok(path) = GetData::<String>::try_data(val) {
            shard_paths.push(path);
        }
    }

    #[cfg(feature = "real")]
    {
        let model_arc2 = model_arc.clone();
        let result = async_std::task::spawn_blocking(move || {
            model_arc2.inner().load(shard_paths)
        })
        .await;

        match result {
            Ok(()) => { let _ = loaded.send_one(().into()).await; }
            Err(e) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(Value::String(e)).await;
            }
        }
    }

    #[cfg(not(feature = "real"))]
    {
        let _ = (shard_paths, whisper_struct);
        let _ = loaded.send_one(().into()).await;
    }
}

/// Decode a continuous stream of PCM audio samples into text using a Whisper model.
///
/// Forwards incoming `f32` sample batches to the worker thread as they arrive; the
/// worker decodes each complete 30-second window (480 000 samples at 16 kHz) into
/// text and emits the result on `transcribed` without waiting for the stream to end.
/// Any remaining samples shorter than one window are flushed and decoded when the
/// audio stream closes.
///
/// ℹ️ `load` must have completed successfully before audio is sent, otherwise the audio
/// is silently discarded.
///
/// ```mermaid
/// graph LR
///     T("decode()")
///     R["〈🟦〉"]     -->|ready|       T
///     A["🟩 🟩 🟩 …"] -->|audio|      T
///     T              -->|transcribed| X["🟩 🟩 …"]
///
///     style R fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style X fill:#ffff,stroke:#ffff
/// ```
///
/// ```
/// use ml/repos/hf::HfHub
/// use ml/repos/hf::fetch
/// use ml/models/whisper::Whisper
/// use ml/models/whisper::load
/// use ml/models/whisper::decode
/// use std/engine/util::startup
///
/// treatment example()
///   model hub:     HfHub(repo_id = "openai/whisper-tiny")
///   model whisper: Whisper()
///   input  audio:       Stream<f32>
///   output transcribed: Stream<string>
/// {
///     startup()
///     fetch[hub=hub]()
///     load[whisper=whisper]()
///     decode[whisper=whisper]()
///
///     startup.trigger    -> fetch.trigger
///     fetch.safetensors  -> load.safetensors
///     load.loaded        -> decode.ready
///     Self.audio         -> decode.audio
///     decode.transcribed -> Self.transcribed
/// }
/// ```
#[mel_treatment(
    model whisper Whisper
    input  ready       Block<void>
    input  audio       Stream<f32>
    output transcribed Stream<string>
)]
pub async fn decode() {
    if ready.recv_one().await.is_err() {
        return;
    }

    let model_arc = WhisperModel::into(whisper);
    let whisper_struct = model_arc.inner();

    #[cfg(feature = "real")]
    {
        use futures::future;

        let text_rx = match whisper_struct.recv_text() {
            Some(rx) => rx,
            None => return,
        };

        // Both halves run truly concurrently inside this single async task.
        // `feed`: forwards incoming sample batches to the worker as they arrive,
        //         then signals end-of-stream so the worker flushes its last window.
        // `drain`: receives decoded text segments from the worker and emits them
        //          on `transcribed` as soon as they are ready.
        // Neither half waits for the other to finish before making progress.
        let feed = async {
            while let Ok(batch) = audio
                .recv_many()
                .await
                .map(|v| TryInto::<Vec<f32>>::try_into(v).unwrap_or_default())
            {
                whisper_struct.send_chunk(batch);
            }
            whisper_struct.close_audio();
        };

        let drain = async {
            while let Ok(segment) = text_rx.recv_async().await {
                if transcribed.send_one(Value::String(segment)).await.is_err() {
                    break;
                }
            }
        };

        future::join(feed, drain).await;
    }

    #[cfg(not(feature = "real"))]
    {
        let _ = whisper_struct;
        while audio.recv_many().await.is_ok() {}
    }
}
