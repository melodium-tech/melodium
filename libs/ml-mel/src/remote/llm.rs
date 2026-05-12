use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
#[cfg(feature = "real")]
use std::sync::Mutex;
use std::sync::{Arc, Weak};

#[cfg(feature = "real")]
use futures::StreamExt;
#[cfg(feature = "real")]
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage, ImageMime},
    LLMProvider,
};

/// Remote LLM provider configuration.
///
/// Holds connection and inference parameters for a remote large language model service.
/// Supports any backend provided by the `llm` crate: OpenAI, Anthropic, Google, Ollama,
/// Groq, Mistral API, DeepSeek, xAI, Cohere, OpenRouter, HuggingFace, Azure OpenAI,
/// and AWS Bedrock.
///
/// - `backend`: provider name — one of `"openai"`, `"anthropic"`, `"ollama"`, `"google"`,
///   `"groq"`, `"mistral"`, `"deepseek"`, `"xai"`, `"cohere"`, `"openrouter"`,
///   `"huggingface"`, `"azure-openai"`, `"aws-bedrock"` (default `"openai"`).
/// - `api_key`: API key for authentication (leave empty for Ollama or unauthenticated endpoints).
/// - `base_url`: override the provider base URL — required for Ollama (e.g. `"http://localhost:11434"`),
///   Azure OpenAI, or custom OpenAI-compatible endpoints.
/// - `model`: model identifier, e.g. `"gpt-4o"`, `"claude-sonnet-4-6"`, `"llama3.2"`.
/// - `system`: system prompt injected at the start of every conversation.
/// - `max_tokens`: maximum tokens to generate per response (default `1024`).
/// - `temperature`: sampling temperature, 0.0–1.0 (default `0.8`).
/// - `top_p`: nucleus sampling cutoff, 0.0–1.0 (default `1.0`).
/// - `timeout`: request timeout in seconds (default `60`).
///
/// ℹ️ Use `RemoteLlm` together with `chat`, `stream`, or `visionChat`.
///
/// ```mel
/// use ml/remote/llm::RemoteLlm
/// use ml/remote/llm::stream
/// use std/engine/util::startup
///
/// treatment example()
///   model llm: RemoteLlm(
///     backend     = "anthropic",
///     api_key     = "sk-ant-...",
///     model       = "claude-sonnet-4-6",
///     system      = "You are a helpful assistant.",
///     max_tokens  = 2048
///   )
///   input  prompt: Stream<string>
///   output token:  Stream<string>
/// {
///     stream[llm=llm]()
///     Self.prompt -> stream.prompt,token -> Self.token
/// }
/// ```
#[mel_model(
    param backend     string  "openai"
    param api_key     string  ""
    param base_url    string  ""
    param model       string  ""
    param system      string  ""
    param max_tokens  u64     1024
    param temperature f32     0.8
    param top_p       f32     1.0
    param timeout     u64     60
    initialize initialize
    shutdown shutdown
)]
pub struct RemoteLlm {
    model: Weak<RemoteLlmModel>,
    #[cfg(feature = "real")]
    provider: Mutex<Option<Arc<Box<dyn LLMProvider>>>>,
}

impl std::fmt::Debug for RemoteLlm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteLlm").finish_non_exhaustive()
    }
}

impl RemoteLlm {
    fn new(model: Weak<RemoteLlmModel>) -> Self {
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
                    eprintln!("[RemoteLlm] invalid backend '{}': {}", backend_str, e);
                    return;
                }
            };

            let api_key = model_ref.get_api_key();
            let base_url = model_ref.get_base_url();
            let model_id = model_ref.get_model();
            let system = model_ref.get_system();
            let max_tokens = model_ref.get_max_tokens() as u32;
            let temperature = model_ref.get_temperature();
            let top_p = model_ref.get_top_p();
            let timeout = model_ref.get_timeout();

            let mut builder = LLMBuilder::new()
                .backend(backend)
                .api_key(api_key)
                .max_tokens(max_tokens)
                .temperature(temperature)
                .top_p(top_p)
                .timeout_seconds(timeout);

            if !base_url.is_empty() {
                builder = builder.base_url(base_url);
            }
            if !model_id.is_empty() {
                builder = builder.model(model_id);
            }
            if !system.is_empty() {
                builder = builder.system(system);
            }

            match builder.build() {
                Ok(p) => {
                    *self.provider.lock().unwrap() = Some(Arc::new(p));
                }
                Err(e) => {
                    eprintln!("[RemoteLlm] failed to build provider: {}", e);
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

/// Send prompts to a remote LLM and receive complete responses.
///
/// For each string received on `prompt`, sends a single-turn chat request to the
/// configured provider and emits the full response text on `response`.  If the
/// request fails, `failed` and `error` are emitted instead.
///
/// ℹ️ `load` is not required — the provider is initialised when the program starts.
/// Use `stream` instead if you want token-by-token output.
///
/// ```mermaid
/// graph LR
///     T("chat()")
///     P["🟩 🟩 …"] -->|prompt|   T
///     T -->|response| R["🟩 🟩 …"]
///     T -->|failed|   F["🟩 🟩 …"]
///     T -->|error|    E["🟩 🟩 …"]
///
///     style P fill:#ffff,stroke:#ffff
///     style R fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
///
/// ```mel
/// use ml/remote/llm::RemoteLlm
/// use ml/remote/llm::chat
///
/// treatment example()
///   model llm: RemoteLlm(backend = "openai", api_key = "sk-...", model = "gpt-4o")
///   input  prompt:   Stream<string>
///   output response: Stream<string>
/// {
///     chat[llm=llm]()
///     Self.prompt -> chat.prompt,response -> Self.response
/// }
/// ```
#[mel_treatment(
    model llm RemoteLlm
    input  prompt   Stream<string>
    output response Stream<string>
    output failed   Stream<void>
    output error    Stream<string>
)]
pub async fn chat() {
    let model_arc = RemoteLlmModel::into(llm);

    while let Ok(val) = prompt.recv_one().await {
        let text = GetData::<String>::try_data(val).unwrap_or_default();

        #[cfg(feature = "real")]
        {
            let maybe_provider = model_arc.inner().provider.lock().unwrap().clone();
            if let Some(provider) = maybe_provider {
                let messages = vec![ChatMessage::user().content(text).build()];
                match provider.chat(&messages).await {
                    Ok(resp) => {
                        let text = resp.text().unwrap_or_default();
                        check!(response.send_one(Value::String(text)).await);
                    }
                    Err(e) => {
                        failed.send_one(().into()).await;
                        error.send_one(Value::String(e.to_string())).await;
                        break;
                    }
                }
            } else {
                failed.send_one(().into()).await;

                error
                    .send_one(Value::String("provider not initialized".into()))
                    .await;
                break;
            }
        }

        #[cfg(not(feature = "real"))]
        {
            let _ = &text;
            let _ = &model_arc;
            check!(
                response
                    .send_one(Value::String("[mock response]".into()))
                    .await
            );
        }
    }
}

/// Stream token-by-token output from a remote LLM.
///
/// For each string received on `prompt`, sends a streaming chat request and emits
/// decoded token strings on `token` as they arrive.  Generation for one prompt ends
/// when the provider closes the stream; the next prompt is then processed.
///
/// If the provider does not support streaming, or if an error occurs, `failed` and
/// `error` are emitted.
///
/// ```mermaid
/// graph LR
///     T("stream()")
///     P["🟩 🟩 …"] -->|prompt| T
///     T -->|token|  K["🟩 🟩 🟩 🟩 …"]
///     T -->|failed| F["🟩 🟩 …"]
///     T -->|error|  E["🟩 🟩 …"]
///
///     style P fill:#ffff,stroke:#ffff
///     style K fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
///
/// ```mel
/// use ml/remote/llm::RemoteLlm
/// use ml/remote/llm::stream
///
/// treatment example()
///   model llm: RemoteLlm(backend = "ollama", base_url = "http://localhost:11434", model = "llama3.2")
///   input  prompt: Stream<string>
///   output token:  Stream<string>
/// {
///     stream[llm=llm]()
///     Self.prompt -> stream.prompt,token -> Self.token
/// }
/// ```
#[mel_treatment(
    model llm RemoteLlm
    input  prompt Stream<string>
    output token  Stream<string>
    output failed Stream<void>
    output error  Stream<string>
)]
pub async fn stream() {
    let model_arc = RemoteLlmModel::into(llm);

    while let Ok(val) = prompt.recv_one().await {
        let text = GetData::<String>::try_data(val).unwrap_or_default();

        #[cfg(feature = "real")]
        {
            let maybe_provider = model_arc.inner().provider.lock().unwrap().clone();
            if let Some(provider) = maybe_provider {
                let messages = vec![ChatMessage::user().content(text).build()];
                match provider.chat_stream(&messages).await {
                    Ok(mut s) => {
                        while let Some(chunk) = s.next().await {
                            match chunk {
                                Ok(t) => {
                                    check!(token.send_one(Value::String(t)).await);
                                }
                                Err(e) => {
                                    failed.send_one(().into()).await;
                                    error.send_one(Value::String(e.to_string())).await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        failed.send_one(().into()).await;
                        error.send_one(Value::String(e.to_string())).await;
                        break;
                    }
                }
            } else {
                failed.send_one(().into()).await;
                error
                    .send_one(Value::String("provider not initialized".into()))
                    .await;
                break;
            }
        }

        #[cfg(not(feature = "real"))]
        {
            let _ = &text;
            let _ = &model_arc;
            check!(token.send_one(Value::String("[mock token]".into())).await);
        }
    }
}

/// Send an image with a text prompt to a remote vision-capable LLM.
///
/// Receives raw image bytes on `image`, the MIME type on `mime` (`"jpeg"`, `"png"`,
/// `"gif"`, or `"webp"`), and a text question on `prompt`.  Sends both to the provider
/// as a two-message chat (image first, then text) and emits the full response on
/// `response`.
///
/// If the provider does not support vision, or if an error occurs, `failed` and
/// `error` are emitted instead.
///
/// ℹ️ Not all backends support image input.  Tested with OpenAI (`gpt-4o`) and
/// Anthropic (`claude-*`).
///
/// ```mermaid
/// graph LR
///     T("visionChat()")
///     I["〈🟦〉"] -->|image|    T
///     M["〈🟨〉"] -->|mime|     T
///     P["〈🟨〉"] -->|prompt|   T
///     T -->|response| R["〈🟨〉"]
///     T -->|failed|   F["〈🟦〉"]
///     T -->|error|    E["〈🟨〉"]
///
///     style I fill:#ffff,stroke:#ffff
///     style M fill:#ffff,stroke:#ffff
///     style P fill:#ffff,stroke:#ffff
///     style R fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    model llm RemoteLlm
    input  image    Block<Vec<byte>>
    input  mime     Block<string>
    input  prompt   Block<string>
    output response Block<string>
    output failed   Block<void>
    output error    Block<string>
)]
pub async fn vision_chat() {
    let model_arc = RemoteLlmModel::into(llm);

    let image_val = match image.recv_one().await {
        Ok(v) => v,
        Err(_) => return,
    };
    let mime_val = match mime.recv_one().await {
        Ok(v) => v,
        Err(_) => return,
    };
    let prompt_val = match prompt.recv_one().await {
        Ok(v) => v,
        Err(_) => return,
    };

    let prompt_text = GetData::<String>::try_data(prompt_val).unwrap_or_default();
    let mime_str = GetData::<String>::try_data(mime_val).unwrap_or_default();

    #[cfg(feature = "real")]
    {
        let image_bytes: Vec<u8> = match image_val {
            Value::Vec(items) => items
                .iter()
                .filter_map(|v| match v {
                    Value::Byte(b) => Some(*b),
                    _ => None,
                })
                .collect(),
            _ => {
                let _ = failed.send_one(().into()).await;
                let _ = error
                    .send_one(Value::String("invalid image value".into()))
                    .await;
                return;
            }
        };

        let image_mime = match mime_str.to_lowercase().as_str() {
            "jpeg" | "jpg" => ImageMime::JPEG,
            "png" => ImageMime::PNG,
            "gif" => ImageMime::GIF,
            "webp" => ImageMime::WEBP,
            other => {
                let _ = failed.send_one(().into()).await;
                let _ = error
                    .send_one(Value::String(format!("unsupported mime type: {}", other)))
                    .await;
                return;
            }
        };

        let messages = vec![
            ChatMessage::user().image(image_mime, image_bytes).build(),
            ChatMessage::user().content(prompt_text).build(),
        ];

        let maybe_provider = model_arc.inner().provider.lock().unwrap().clone();
        if let Some(provider) = maybe_provider {
            match provider.chat(&messages).await {
                Ok(resp) => {
                    let text = resp.text().unwrap_or_default();
                    let _ = response.send_one(Value::String(text)).await;
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
        let _ = (&model_arc, &prompt_text, &mime_str, &image_val);
        let _ = response
            .send_one(Value::String("[mock vision response]".into()))
            .await;
    }
}
