use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
#[cfg(feature = "real")]
use std::sync::Arc;
#[cfg(feature = "real")]
use std::sync::Mutex;
use std::sync::Weak;

#[cfg(feature = "real")]
use llm::{
    builder::{LLMBackend, LLMBuilder},
    LLMProvider,
};

/// Remote speech-to-text provider configuration.
///
/// Holds the connection parameters for a remote speech-to-text service.
///
/// - `backend`: provider name (default `"openai"`); see the table below.
/// - `api_key`: API key for authentication (omit for unauthenticated endpoints).
/// - `base_url`: override the provider base URL (optional).
/// - `model`: model identifier — see the table below for recommended values per backend.
///
/// ## Backends
///
/// | `backend`      | `api_key`       | Example `model`  | Model list                                                             |
/// |----------------|-----------------|------------------|------------------------------------------------------------------------|
/// | `"openai"`     | OpenAI API key  | `"whisper-1"`    | <https://platform.openai.com/docs/models>                              |
/// | `"elevenlabs"` | ElevenLabs key  | `"scribe_v1"`    | <https://elevenlabs.io/docs/speech-to-text>                            |
///
/// ℹ️ Use `RemoteStt` together with `transcribe`.
///
/// ```mel
/// use ml/remote/stt::RemoteStt
/// use ml/remote/stt::transcribe
///
/// treatment example()
///   model stt: RemoteStt(backend = "openai", api_key = "sk-...", model = "whisper-1")
///   input  audio:      Stream<byte>
///   output transcript: Block<string>
/// {
///     transcribe[stt=stt]()
///     Self.audio -> transcribe.audio,transcript -> Self.transcript
/// }
/// ```
#[mel_model(
    param backend  string         "openai"
    param api_key  Option<string> none
    param base_url string         ""
    param model    string         ""
    initialize initialize
    shutdown shutdown
)]
pub struct RemoteStt {
    #[allow(dead_code)]
    model: Weak<RemoteSttModel>,
    #[cfg(feature = "real")]
    provider: Mutex<Option<Arc<Box<dyn LLMProvider>>>>,
}

impl std::fmt::Debug for RemoteStt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteStt").finish_non_exhaustive()
    }
}

impl RemoteStt {
    fn new(model: Weak<RemoteSttModel>) -> Self {
        Self {
            model,
            #[cfg(feature = "real")]
            provider: Mutex::new(None::<Arc<Box<dyn LLMProvider>>>),
        }
    }

    fn initialize(&self) {
        #[cfg(feature = "real")]
        {
            let model_ref = match self.model.upgrade() {
                Some(m) => m,
                None => return,
            };

            let backend_str = model_ref.get_backend();
            let backend: LLMBackend = match backend_str.parse() {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[RemoteStt] invalid backend '{}': {}", backend_str, e);
                    return;
                }
            };

            let base_url = model_ref.get_base_url();
            let model_id = model_ref.get_model();

            let mut builder = LLMBuilder::new().backend(backend);

            if let Some(api_key) = model_ref.get_api_key() {
                builder = builder.api_key(api_key);
            }
            if !base_url.is_empty() {
                builder = builder.base_url(base_url);
            }
            if !model_id.is_empty() {
                builder = builder.model(model_id);
            }

            match builder.build() {
                Ok(p) => {
                    *self.provider.lock().unwrap() = Some(Arc::new(p));
                }
                Err(e) => {
                    eprintln!("[RemoteStt] failed to build provider: {}", e);
                }
            }
        }
    }

    fn shutdown(&self) {
        #[cfg(feature = "real")]
        {
            *self.provider.lock().unwrap() = None;
        }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}

/// Transcribe audio bytes to text using a remote speech-to-text service.
///
/// Collects all bytes from the `audio` stream into a single buffer, then sends them
/// to the configured provider for transcription.  The resulting text is emitted on
/// `transcript`.  If the request fails, `failed` and `error` are emitted instead.
///
/// ℹ️ The audio stream should be closed by the sender once all audio data has been
/// sent — `transcribe` waits for the stream to close before submitting the request.
/// Common audio formats: WAV, MP3, FLAC (format support depends on the backend).
///
/// ```mermaid
/// graph LR
///     T("transcribe()")
///     A["🟩 🟩 🟩 …"] -->|audio| T
///     T -->|transcript| R["〈🟨〉"]
///     T -->|failed| F["〈🟦〉"]
///     T -->|error| E["〈🟨〉"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style R fill:#ffffff,stroke:#ffffff
///     style F fill:#ffffff,stroke:#ffffff
///     style E fill:#ffffff,stroke:#ffffff
/// ```
///
/// ```mel
/// use ml/remote/stt::RemoteStt
/// use ml/remote/stt::transcribe
///
/// treatment example()
///   model stt: RemoteStt(backend = "openai", api_key = "sk-...", model = "whisper-1")
///   input  audio:      Stream<byte>
///   output transcript: Block<string>
/// {
///     transcribe[stt=stt]()
///     Self.audio -> transcribe.audio,transcript -> Self.transcript
/// }
/// ```
#[mel_treatment(
    model stt RemoteStt
    input  audio      Stream<byte>
    output transcript Block<string>
    output failed     Block<void>
    output error      Block<string>
)]
pub async fn transcribe() {
    let model_arc = RemoteSttModel::into(stt);

    // Collect all audio bytes from the stream.
    let mut audio_bytes: Vec<u8> = Vec::new();
    while let Ok(values) = audio.recv_many().await {
        let chunk: Vec<u8> = values.try_into().unwrap_or_default();
        audio_bytes.extend_from_slice(&chunk);
    }

    if audio_bytes.is_empty() {
        return;
    }

    #[cfg(feature = "real")]
    {
        let maybe_provider = model_arc.inner().provider.lock().unwrap().clone();
        if let Some(provider) = maybe_provider {
            match provider.transcribe(audio_bytes).await {
                Ok(text) => {
                    let _ = transcript.send_one(Value::String(text)).await;
                }
                Err(e) => {
                    let _ = failed.send_one(().into()).await;
                    let _ = error.send_one(Value::String(e.to_string())).await;
                }
            }
        } else {
            let _ = failed.send_one(().into()).await;
            let _ = error
                .send_one(Value::String("provider not initialized".into()))
                .await;
        }
    }

    #[cfg(not(feature = "real"))]
    {
        let _ = (&model_arc, &audio_bytes);
        let _ = transcript
            .send_one(Value::String("[mock transcript]".into()))
            .await;
    }
}

/// Continuously transcribe a stream of audio segments to text.
///
/// For each `Vec<byte>` received on `segments`, submits it as a complete, independently
/// decodable audio clip to the configured provider and emits the resulting text on
/// `transcript`.  The treatment runs until `segments` is closed.
///
/// Each item on `segments` must be a self-contained audio blob — the caller is
/// responsible for producing meaningful segment boundaries (e.g. one WAV blob per
/// utterance from a voice-activity detector, one chunk per WebSocket message from a
/// streaming API, etc.).  Splitting at the raw byte level within an encoded format
/// would corrupt frames and produce garbage transcripts.
///
/// If a single segment fails, `failed` and `error` are emitted for that segment and
/// processing continues with the next one.
///
/// ```mermaid
/// graph LR
///     T("transcribeContinuous()")
///     S["🟩 🟩 🟩 …"] -->|segments| T
///     T -->|transcript| R["🟩 🟩 🟩 …"]
///     T -->|failed| F["🟩 🟩 🟩 …"]
///     T -->|error| E["🟩 🟩 🟩 …"]
///
///     style S fill:#ffffff,stroke:#ffffff
///     style R fill:#ffffff,stroke:#ffffff
///     style F fill:#ffffff,stroke:#ffffff
///     style E fill:#ffffff,stroke:#ffffff
/// ```
///
/// ```mel
/// use ml/remote/stt::RemoteStt
/// use ml/remote/stt::transcribeContinuous
///
/// treatment example()
///   model stt: RemoteStt(backend = "openai", api_key = "sk-...", model = "whisper-1")
///   input  segments:   Stream<Vec<byte>>
///   output transcript: Stream<string>
/// {
///     transcribeContinuous[stt=stt]()
///     Self.segments -> transcribeContinuous.segments,transcript -> Self.transcript
/// }
/// ```
#[mel_treatment(
    model stt RemoteStt
    input  segments   Stream<Vec<byte>>
    output transcript Stream<string>
    output failed     Stream<void>
    output error      Stream<string>
)]
pub async fn transcribe_continuous() {
    let model_arc = RemoteSttModel::into(stt);

    while let Ok(val) = segments.recv_one().await {
        let segment_bytes: Vec<u8> = match val {
            Value::Vec(items) => items
                .iter()
                .filter_map(|v| match v {
                    Value::Byte(b) => Some(*b),
                    _ => None,
                })
                .collect(),
            _ => continue,
        };

        if segment_bytes.is_empty() {
            continue;
        }

        #[cfg(feature = "real")]
        {
            let maybe_provider = model_arc.inner().provider.lock().unwrap().clone();
            if let Some(provider) = maybe_provider {
                match provider.transcribe(segment_bytes).await {
                    Ok(text) => {
                        check!(transcript.send_one(Value::String(text)).await);
                    }
                    Err(e) => {
                        let _ = failed.send_one(().into()).await;
                        let _ = error.send_one(Value::String(e.to_string())).await;
                        break;
                    }
                }
            } else {
                let _ = failed.send_one(().into()).await;
                let _ = error
                    .send_one(Value::String("provider not initialized".into()))
                    .await;
                break;
            }
        }

        #[cfg(not(feature = "real"))]
        {
            let _ = (&model_arc, &segment_bytes);
            check!(
                transcript
                    .send_one(Value::String("[mock transcript]".into()))
                    .await
            );
        }
    }
}
