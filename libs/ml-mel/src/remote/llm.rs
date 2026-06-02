use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
#[cfg(feature = "real")]
use std::sync::Arc;
#[cfg(feature = "real")]
use std::sync::Mutex;
use std::sync::Weak;

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
///
/// - `backend`: provider name (default `"mistral"`); see the table below.
/// - `api_key`: API key for authentication (omit for Ollama or unauthenticated endpoints).
/// - `base_url`: override the provider base URL (required for Ollama, Azure OpenAI,
///   or custom OpenAI-compatible endpoints; see the table below).
/// - `model`: model identifier; see the table below for recommended values per backend.
/// - `system`: system prompt injected at the start of every conversation.
/// - `max_tokens`: maximum tokens to generate per response (omit to use the backend default).
/// - `temperature`: sampling temperature from 0.0 to 2.0 (omit to use the backend default; some
///   backends reject `temperature` and `top_p` being set simultaneously).
/// - `top_p`: nucleus sampling cutoff from 0.0 to 1.0 (omit to use the backend default).
/// - `timeout`: request timeout in seconds (omit to use the backend default).
///
/// ## Backends
///
/// | `backend`        | `api_key`         | `base_url`                              | Example `model`               | Model list                                                                                  |
/// |------------------|-------------------|-----------------------------------------|-------------------------------|---------------------------------------------------------------------------------------------|
/// | `"mistral"`      | Mistral API key   | *(built-in)*                            | `"mistral-small-latest"`      | <https://docs.mistral.ai/getting-started/models/models_overview/>                          |
/// | `"openai"`       | OpenAI API key    | *(built-in)*                            | `"gpt-4.1-nano"`              | <https://platform.openai.com/docs/models>                                                   |
/// | `"anthropic"`    | Anthropic API key | *(built-in)*                            | `"claude-sonnet-4-6"`         | <https://docs.anthropic.com/en/docs/about-claude/models/all-models>                         |
/// | `"google"`       | Google API key    | *(built-in)*                            | `"gemini-2.5-flash"`          | <https://ai.google.dev/gemini-api/docs/models>                                              |
/// | `"groq"`         | Groq API key      | *(built-in)*                            | `"llama-3.3-70b-versatile"`   | <https://console.groq.com/docs/models>                                                      |
/// | `"deepseek"`     | DeepSeek API key  | *(built-in)*                            | `"deepseek-chat"`             | <https://api-docs.deepseek.com/quick_start/pricing>                                         |
/// | `"xai"`          | xAI API key       | *(built-in)*                            | `"grok-2-latest"`             | <https://docs.x.ai/docs/models>                                                             |
/// | `"cohere"`       | Cohere API key    | *(built-in)*                            | `"command-a-03-2025"`         | <https://docs.cohere.com/docs/models>                                                       |
/// | `"openrouter"`   | OpenRouter key    | *(built-in)*                            | `"provider/model-name"`       | <https://openrouter.ai/models>                                                              |
/// | `"huggingface"`  | HF token          | *(built-in)*                            | any HF Inference model ID     | <https://huggingface.co/models?pipeline_tag=text-generation&inference=warm>                 |
/// | `"ollama"`       | *(omit)*          | `"http://localhost:11434"`              | `"llama3.2"`                  | <https://ollama.com/library>                                                                |
/// | `"azure-openai"` | Azure API key     | `"https://<resource>.openai.azure.com"` | `"gpt-4o"`                    | <https://learn.microsoft.com/azure/ai-services/openai/concepts/models>                      |
/// | `"aws-bedrock"`  | *(AWS env vars)*  | *(built-in)*                            | Bedrock model ARN or ID       | <https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html>                |
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
///     backend    = "mistral",
///     api_key    = "...",
///     model      = "mistral-small-latest",
///     system     = "You are a helpful assistant.",
///     max_tokens = 2048
///   )
///   input  prompt: Stream<string>
///   output token:  Stream<string>
/// {
///     stream[llm=llm]()
///     Self.prompt -> stream.prompt,token -> Self.token
/// }
/// ```
#[mel_model(
    param backend     string       "mistral"
    param api_key     Option<string> none
    param base_url    string       ""
    param model       string       ""
    param system      string       ""
    param max_tokens  Option<u64>  none
    param temperature Option<f32>  none
    param top_p       Option<f32>  none
    param timeout     Option<u64>  none
    initialize initialize
    shutdown shutdown
)]
pub struct RemoteLlm {
    #[allow(dead_code)]
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

            let base_url = model_ref.get_base_url();
            let model_id = model_ref.get_model();
            let system = model_ref.get_system();

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
            if !system.is_empty() {
                builder = builder.system(system);
            }
            if let Some(max_tokens) = model_ref.get_max_tokens() {
                builder = builder.max_tokens(max_tokens as u32);
            }
            if let Some(temperature) = model_ref.get_temperature() {
                builder = builder.temperature(temperature);
            }
            if let Some(top_p) = model_ref.get_top_p() {
                builder = builder.top_p(top_p);
            }
            if let Some(timeout) = model_ref.get_timeout() {
                builder = builder.timeout_seconds(timeout);
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
/// ℹ️ `load` is not required; the provider is initialised when the program starts.
/// Use `stream` instead if you want token-by-token output.
///
/// ```mermaid
/// graph LR
///     T("chat()")
///     P["🟩 🟩 …"] -->|prompt| T
///     T -->|response| R["🟩 🟩 …"]
///     T -->|failed| F["🟩 🟩 …"]
///     T -->|error| E["🟩 🟩 …"]
///
///     style P fill:#ffffff,stroke:#ffffff
///     style R fill:#ffffff,stroke:#ffffff
///     style F fill:#ffffff,stroke:#ffffff
///     style E fill:#ffffff,stroke:#ffffff
/// ```
///
/// ```mel
/// use ml/remote/llm::RemoteLlm
/// use ml/remote/llm::chat
///
/// treatment example()
///   model llm: RemoteLlm(backend = "mistral", api_key = "...", model = "mistral-small-latest")
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
///     T -->|token| K["🟩 🟩 🟩 🟩 …"]
///     T -->|failed| F["🟩 🟩 …"]
///     T -->|error| E["🟩 🟩 …"]
///
///     style P fill:#ffffff,stroke:#ffffff
///     style K fill:#ffffff,stroke:#ffffff
///     style F fill:#ffffff,stroke:#ffffff
///     style E fill:#ffffff,stroke:#ffffff
/// ```
///
/// ```mel
/// use ml/remote/llm::RemoteLlm
/// use ml/remote/llm::stream
///
/// treatment example()
///   model llm: RemoteLlm(backend = "mistral", api_key = "...", model = "mistral-small-latest")
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
                                    let _ = failed.send_one(().into()).await;
                                    let _ = error.send_one(Value::String(e.to_string())).await;
                                    break;
                                }
                            }
                        }
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
///     I["〈🟦〉"] -->|image| T
///     M["〈🟨〉"] -->|mime| T
///     P["〈🟨〉"] -->|prompt| T
///     T -->|response| R["〈🟨〉"]
///     T -->|failed| F["〈🟦〉"]
///     T -->|error| E["〈🟨〉"]
///
///     style I fill:#ffffff,stroke:#ffffff
///     style M fill:#ffffff,stroke:#ffffff
///     style P fill:#ffffff,stroke:#ffffff
///     style R fill:#ffffff,stroke:#ffffff
///     style F fill:#ffffff,stroke:#ffffff
///     style E fill:#ffffff,stroke:#ffffff
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
