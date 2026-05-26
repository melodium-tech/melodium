use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
use std::collections::VecDeque;
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

/// Remote text-to-speech provider configuration.
///
/// Holds the connection parameters for a remote text-to-speech service.
///
/// - `backend`: provider name (default `"elevenlabs"`); see the table below.
/// - `api_key`: API key for authentication (omit for unauthenticated endpoints).
/// - `base_url`: override the provider base URL (optional).
/// - `model`: model identifier — see the table below for recommended values per backend.
/// - `voice`: voice identifier (required for ElevenLabs; omit for other backends).
///
/// ## Backends
///
/// | `backend`      | `api_key`      | Example `model`            | `voice`             | Model & voice list                                          |
/// |----------------|----------------|----------------------------|---------------------|-------------------------------------------------------------|
/// | `"elevenlabs"` | ElevenLabs key | `"eleven_multilingual_v2"` | ElevenLabs voice ID | <https://elevenlabs.io/docs/text-to-speech>                 |
///
/// ℹ️ Use `RemoteTts` together with `synthesize`.
///
/// ```mel
/// use ml/remote/tts::RemoteTts
/// use ml/remote/tts::synthesize
///
/// treatment example()
///   model tts: RemoteTts(
///     backend  = "elevenlabs",
///     api_key  = "...",
///     model    = "eleven_multilingual_v2",
///     voice    = "JBFqnCBsd6RMkjVDRZzb"
///   )
///   input  text:  Stream<string>
///   output audio: Stream<byte>
/// {
///     synthesize[tts=tts]()
///     Self.text -> synthesize.text,audio -> Self.audio
/// }
/// ```
#[mel_model(
    param backend  string         "elevenlabs"
    param api_key  Option<string> none
    param base_url string         ""
    param model    string         ""
    param voice    string         ""
    initialize initialize
    shutdown shutdown
)]
pub struct RemoteTts {
    #[allow(dead_code)]
    model: Weak<RemoteTtsModel>,
    #[cfg(feature = "real")]
    provider: Mutex<Option<Arc<Box<dyn LLMProvider>>>>,
}

impl std::fmt::Debug for RemoteTts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteTts").finish_non_exhaustive()
    }
}

impl RemoteTts {
    fn new(model: Weak<RemoteTtsModel>) -> Self {
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
                    eprintln!("[RemoteTts] invalid backend '{}': {}", backend_str, e);
                    return;
                }
            };

            let base_url = model_ref.get_base_url();
            let model_id = model_ref.get_model();
            let voice = model_ref.get_voice();

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
            if !voice.is_empty() {
                builder = builder.voice(voice);
            }

            match builder.build() {
                Ok(p) => {
                    *self.provider.lock().unwrap() = Some(Arc::new(p));
                }
                Err(e) => {
                    eprintln!("[RemoteTts] failed to build provider: {}", e);
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

/// Synthesize speech audio from text using a remote text-to-speech service.
///
/// For each string received on `text`, sends a synthesis request to the configured
/// provider and emits the resulting audio bytes on `audio` (MP3 format for ElevenLabs).
/// If the request fails, `failed` and `error` are emitted instead.
///
/// ```mermaid
/// graph LR
///     T("synthesize()")
///     X["🟩 🟩 …"] -->|text|  T
///     T -->|audio|  A["🟩 🟩 🟩 🟩 …"]
///     T -->|failed| F["🟩 🟩 …"]
///     T -->|error|  E["🟩 🟩 …"]
///
///     style X fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
///
/// ```mel
/// use ml/remote/tts::RemoteTts
/// use ml/remote/tts::synthesize
///
/// treatment example()
///   model tts: RemoteTts(
///     backend = "elevenlabs",
///     api_key = "...",
///     voice   = "JBFqnCBsd6RMkjVDRZzb"
///   )
///   input  text:  Stream<string>
///   output audio: Stream<byte>
/// {
///     synthesize[tts=tts]()
///     Self.text -> synthesize.text,audio -> Self.audio
/// }
/// ```
#[mel_treatment(
    model tts RemoteTts
    input  text   Stream<string>
    output audio  Stream<byte>
    output failed Stream<void>
    output error  Stream<string>
)]
pub async fn synthesize() {
    let model_arc = RemoteTtsModel::into(tts);

    while let Ok(val) = text.recv_one().await {
        let text_str = GetData::<String>::try_data(val).unwrap_or_default();

        #[cfg(feature = "real")]
        {
            let maybe_provider = model_arc.inner().provider.lock().unwrap().clone();
            if let Some(provider) = maybe_provider {
                match provider.speech(&text_str).await {
                    Ok(bytes) => {
                        let batch = TransmissionValue::Byte(VecDeque::from(bytes));
                        check!(audio.send_many(batch).await);
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
            let _ = (&model_arc, &text_str);
            // emit a single zero byte as a stub
            check!(
                audio
                    .send_many(TransmissionValue::Byte(VecDeque::from(vec![0u8])))
                    .await
            );
        }
    }
}
