use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::mistral::{Config, Model};
use candle_transformers::generation::LogitsProcessor;
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
use std::sync::Weak;
use tokenizers::Tokenizer;

// A request sent to the inference worker thread.
// The worker sends back individual token strings through `reply`; dropping the
// sender signals that generation is finished.
#[cfg(feature = "real")]
struct InferRequest {
    prompt: String,
    reply: flume::Sender<String>,
}

/// Mistral large language model configuration.
///
/// Holds the architecture and inference hyper-parameters for a Mistral model.  Weights and
/// tokenizer are not embedded here — use an `HfHub` model together with `fetch` and `load`
/// to supply them at runtime, then call `generate` to run inference.
///
/// Architecture parameters (defaults match Mistral-7B-v0.1):
/// - `vocab_size`: vocabulary size.
/// - `hidden_size`: hidden dimension.
/// - `intermediate_size`: feed-forward intermediate dimension.
/// - `num_hidden_layers`: number of transformer layers.
/// - `num_attention_heads`: number of attention heads.
/// - `num_key_value_heads`: number of key/value heads (grouped-query attention).
/// - `max_position_embeddings`: maximum sequence length.
/// - `rms_norm_eps`: RMS-norm epsilon.
/// - `rope_theta`: rotary positional embedding theta.
/// - `sliding_window`: sliding window attention size (`0` disables it).
///
/// Inference parameters:
/// - `temperature`: sampling temperature (`0.0` selects the highest-probability token).
/// - `top_p`: nucleus sampling cutoff (`0.0` disables nucleus sampling).
/// - `repeat_penalty`: logit penalty applied to recently seen tokens.
/// - `repeat_last_n`: number of past tokens considered for the repeat penalty.
/// - `max_new_tokens`: maximum tokens generated per prompt.
///
/// ℹ️ Use `Mistral` together with `HfHub`, `fetch`, `load`, and `generate`.
/// `load` must complete successfully before `generate` will produce output.
///
/// ```
/// use ml/repos/hf::HfHub
/// use ml/repos/hf::fetch
/// use ml/models/mistral::Mistral
/// use ml/models/mistral::load
/// use ml/models/mistral::generate
/// use std/engine/util::startup
///
/// treatment example()
///   model hub:     HfHub(repo_id = "mistralai/Mistral-7B-v0.1")
///   model mistral: Mistral(temperature = 0.7, max_new_tokens = 256)
///   input  prompt:    Stream<string>
///   output generated: Stream<string>
/// {
///     startup()
///     fetch[hub=hub]()
///     load[mistral=mistral]()
///     generate[mistral=mistral]()
///
///     startup.trigger    -> fetch.trigger
///     fetch.safetensors  -> load.safetensors
///     fetch.tokenizer    -> load.tokenizer
///     Self.prompt        -> generate.prompt
///     generate.generated -> Self.generated
/// }
/// ```
#[mel_model(
    param vocab_size              u64     32000
    param hidden_size             u64     4096
    param intermediate_size       u64     14336
    param num_hidden_layers       u64     32
    param num_attention_heads     u64     32
    param num_key_value_heads     u64     8
    param max_position_embeddings u64     32768
    param rms_norm_eps            f64     0.00001
    param rope_theta              f64     10000.0
    param sliding_window          u64     4096
    param temperature             f64     0.8
    param top_p                   f64     0.9
    param repeat_penalty          f32     1.1
    param repeat_last_n           u64     64
    param max_new_tokens          u64     512
    shutdown shutdown
)]
#[derive(Debug)]
pub struct Mistral {
    model: Weak<MistralModel>,
    // Sending end of the request queue. The worker thread owns the receiving end.
    // None until load() is called successfully.
    #[cfg(feature = "real")]
    request_tx: std::sync::Mutex<Option<flume::Sender<InferRequest>>>,
}

impl Mistral {
    fn new(model: Weak<MistralModel>) -> Self {
        Self {
            model,
            #[cfg(feature = "real")]
            request_tx: std::sync::Mutex::new(None),
        }
    }

    fn shutdown(&self) {
        // Dropping the sender closes the channel; the worker loop exits cleanly.
        #[cfg(feature = "real")]
        {
            *self.request_tx.lock().unwrap() = None;
        }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    /// Load weights from `shard_paths` and a tokenizer from `tokenizer_path`, then start
    /// the inference worker thread.  Returns `Err` with a description on any failure.
    /// Calling `load` a second time replaces the previous worker (the old one drains and exits).
    #[cfg(feature = "real")]
    pub fn load(
        &self,
        shard_paths: Vec<String>,
        tokenizer_path: String,
    ) -> Result<(), String> {
        let model_ref = self.model.upgrade().ok_or("model dropped")?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| e.to_string())?;

        let sliding_window_val = model_ref.get_sliding_window();
        let config = Config {
            vocab_size:              model_ref.get_vocab_size() as usize,
            hidden_size:             model_ref.get_hidden_size() as usize,
            intermediate_size:       model_ref.get_intermediate_size() as usize,
            num_hidden_layers:       model_ref.get_num_hidden_layers() as usize,
            num_attention_heads:     model_ref.get_num_attention_heads() as usize,
            head_dim:                None,
            num_key_value_heads:     model_ref.get_num_key_value_heads() as usize,
            hidden_act:              candle_nn::Activation::Silu,
            max_position_embeddings: model_ref.get_max_position_embeddings() as usize,
            rms_norm_eps:            model_ref.get_rms_norm_eps(),
            rope_theta:              model_ref.get_rope_theta(),
            sliding_window:          if sliding_window_val == 0 {
                                         None
                                     } else {
                                         Some(sliding_window_val as usize)
                                     },
            use_flash_attn: false,
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

        let candle_model = Model::new(&config, vb).map_err(|e| e.to_string())?;

        let temperature    = model_ref.get_temperature();
        let top_p          = model_ref.get_top_p();
        let repeat_penalty = model_ref.get_repeat_penalty();
        let repeat_last_n  = model_ref.get_repeat_last_n() as usize;
        let max_new_tokens = model_ref.get_max_new_tokens() as usize;

        // Unbounded channel: callers enqueue immediately, the worker drains sequentially.
        let (tx, rx) = flume::unbounded::<InferRequest>();

        // The worker thread owns the model and the receiving end of the channel.
        // It processes requests one at a time, resetting the KV cache between them.
        std::thread::spawn(move || {
            worker_loop(
                candle_model,
                tokenizer,
                rx,
                temperature,
                top_p,
                repeat_penalty,
                repeat_last_n,
                max_new_tokens,
            );
        });

        // Replacing the sender drops the old one, which closes the old channel and
        // causes the previous worker (if any) to drain and exit naturally.
        *self.request_tx.lock().unwrap() = Some(tx);
        Ok(())
    }

    /// Enqueue a generation request and return a receiver for the token stream.
    /// Returns `None` if the model has not been loaded yet.
    #[cfg(feature = "real")]
    pub fn enqueue(&self, prompt: String) -> Option<flume::Receiver<String>> {
        let (reply_tx, reply_rx) = flume::unbounded();
        let tx = self.request_tx.lock().unwrap();
        tx.as_ref()?.send(InferRequest { prompt, reply: reply_tx }).ok()?;
        Some(reply_rx)
    }
}

// Runs on a dedicated OS thread; never touches the async executor.
// Processes one request at a time, resets KV cache between requests.
#[cfg(feature = "real")]
fn worker_loop(
    mut model: Model,
    tokenizer: Tokenizer,
    rx: flume::Receiver<InferRequest>,
    temperature: f64,
    top_p: f64,
    repeat_penalty: f32,
    repeat_last_n: usize,
    max_new_tokens: usize,
) {
    let device = Device::Cpu;
    let eos_token = tokenizer.token_to_id("</s>").unwrap_or(u32::MAX);

    for req in rx.iter() {
        // Always reset before a new request so previous KV state is gone.
        model.clear_kv_cache();

        let tokens = match tokenizer
            .encode(req.prompt.as_str(), true)
            .map(|e| e.get_ids().to_vec())
        {
            Ok(t) => t,
            Err(_) => continue,
        };

        let mut logits_processor = LogitsProcessor::new(
            42,
            if temperature == 0.0 { None } else { Some(temperature) },
            if top_p == 0.0 { None } else { Some(top_p) },
        );

        let apply_repeat_penalty = repeat_penalty != 1.0;
        let mut all_tokens = tokens.clone();
        let mut seqlen_offset = 0usize;

        // Forward the whole prompt in one shot to fill the KV cache.
        let prompt_tensor = match Tensor::new(tokens.as_slice(), &device)
            .and_then(|t| t.unsqueeze(0))
        {
            Ok(t) => t,
            Err(_) => continue,
        };

        let logits = match model.forward(&prompt_tensor, seqlen_offset) {
            Ok(l) => l,
            Err(_) => continue,
        };
        seqlen_offset += all_tokens.len();

        let mut next_token = match step(
            logits,
            &all_tokens,
            apply_repeat_penalty,
            repeat_penalty,
            repeat_last_n,
            &mut logits_processor,
        ) {
            Some(t) => t,
            None => continue,
        };

        emit_token(&tokenizer, next_token, &req.reply);
        all_tokens.push(next_token);

        for _ in 1..max_new_tokens {
            if next_token == eos_token {
                break;
            }

            let input = match Tensor::new(&[next_token], &device).and_then(|t| t.unsqueeze(0)) {
                Ok(t) => t,
                Err(_) => break,
            };

            let logits = match model.forward(&input, seqlen_offset) {
                Ok(l) => l,
                Err(_) => break,
            };
            seqlen_offset += 1;

            next_token = match step(
                logits,
                &all_tokens,
                apply_repeat_penalty,
                repeat_penalty,
                repeat_last_n,
                &mut logits_processor,
            ) {
                Some(t) => t,
                None => break,
            };

            emit_token(&tokenizer, next_token, &req.reply);
            all_tokens.push(next_token);
        }
        // Dropping `req.reply` here closes the receiver on the async side.
    }
}

#[cfg(feature = "real")]
fn step(
    logits: Tensor,
    all_tokens: &[u32],
    apply_repeat_penalty: bool,
    repeat_penalty: f32,
    repeat_last_n: usize,
    logits_processor: &mut LogitsProcessor,
) -> Option<u32> {
    let logits = logits.squeeze(0).and_then(|l| l.squeeze(0)).ok()?;
    let logits = if apply_repeat_penalty && all_tokens.len() >= repeat_last_n {
        let start = all_tokens.len().saturating_sub(repeat_last_n);
        candle_transformers::utils::apply_repeat_penalty(
            &logits,
            repeat_penalty,
            &all_tokens[start..],
        )
        .ok()?
    } else {
        logits
    };
    logits_processor.sample(&logits).ok()
}

#[cfg(feature = "real")]
fn emit_token(tokenizer: &Tokenizer, token: u32, reply: &flume::Sender<String>) {
    if let Ok(word) = tokenizer.decode(&[token], true) {
        if !word.is_empty() {
            // If the receiver has been dropped the treatment has gone away; ignore.
            let _ = reply.send(word);
        }
    }
}

/// Load weights and tokenizer into a Mistral model.
///
/// Collects all `.safetensors` shard paths from `safetensors` (stream closes when all shards
/// have been emitted), then waits for the single tokenizer path on `tokenizer`.  Once both are
/// received, memory-maps the weight shards and starts the inference worker thread inside the
/// `Mistral` model.
///
/// `loaded` is emitted when the model is ready to accept prompts.  If any step fails —
/// file not found, incompatible weights, tokenizer parse error — `failed` and `error` are
/// emitted instead and `loaded` is never sent.
///
/// ℹ️ Wire `safetensors` and `tokenizer` directly from a `fetch` treatment.
///
/// ⚠️ `generate` will silently drop prompts until `load` has successfully completed.
///
/// ```mermaid
/// graph LR
///     T("load()")
///     S["🟩 🟩 🟩 …"] -->|safetensors| T
///     K["〈🟨〉"]       -->|tokenizer|   T
///     T -->|loaded| L["〈🟦〉"]
///     T -->|failed| F["〈🟦〉"]
///     T -->|error|  E["〈🟨〉"]
///
///     style S fill:#ffff,stroke:#ffff
///     style K fill:#ffff,stroke:#ffff
///     style L fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
///
/// ```
/// use ml/repos/hf::HfHub
/// use ml/repos/hf::fetch
/// use ml/models/mistral::Mistral
/// use ml/models/mistral::load
/// use std/engine/util::startup
///
/// treatment example()
///   model hub:     HfHub(repo_id = "mistralai/Mistral-7B-v0.1")
///   model mistral: Mistral()
/// {
///     startup()
///     fetch[hub=hub]()
///     load[mistral=mistral]()
///
///     startup.trigger   -> fetch.trigger
///     fetch.safetensors -> load.safetensors
///     fetch.tokenizer   -> load.tokenizer
/// }
/// ```
#[mel_treatment(
    model mistral Mistral
    input  safetensors Stream<string>
    input  tokenizer   Block<string>
    output loaded      Block<void>
    output failed      Block<void>
    output error       Block<string>
)]
pub async fn load() {
    let model_arc = MistralModel::into(mistral);
    let mistral_struct = model_arc.inner();

    // Collect all shard paths from the stream.
    let mut shard_paths: Vec<String> = Vec::new();
    while let Ok(val) = safetensors.recv_one().await {
        if let Ok(path) = GetData::<String>::try_data(val) {
            shard_paths.push(path);
        }
    }

    // Receive the single tokenizer path.
    let tokenizer_path = match tokenizer.recv_one().await {
        Ok(val) => match GetData::<String>::try_data(val) {
            Ok(p) => p,
            Err(_) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(Value::String("invalid tokenizer path value".into())).await;
                return;
            }
        },
        Err(_) => {
            let _ = failed.send_one(().into()).await;
            let _ = error.send_one(Value::String("tokenizer path not received".into())).await;
            return;
        }
    };

    #[cfg(feature = "real")]
    {
        // load() is sync but does heavy I/O (mmap) and CPU work (model init);
        // run it on a blocking thread so the async executor is not stalled.
        // Clone the Arc so the model stays alive inside the blocking closure.
        let model_arc2 = model_arc.clone();
        let result = async_std::task::spawn_blocking(move || {
            model_arc2.inner().load(shard_paths, tokenizer_path)
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
        let _ = (shard_paths, tokenizer_path, mistral_struct);
        let _ = loaded.send_one(().into()).await;
    }
}

/// Generate text from a Mistral model, one token fragment per stream item.
///
/// For each string received on `prompt`, enqueues an inference request and emits the
/// decoded token strings on `generated` as they arrive — one string per token.  Generation
/// for a single prompt ends when the model produces `</s>` or when `max_new_tokens` is
/// reached.  The next prompt is then dequeued.
///
/// Each concurrent track gets its own independent reply channel and is processed in
/// arrival order on a dedicated OS thread, so the async executor is never blocked during
/// inference.
///
/// ℹ️ `load` must have completed successfully before any prompt is sent, otherwise prompts
/// are silently discarded.
///
/// ```mermaid
/// graph LR
///     T("generate()")
///     P["🟩 🟩 …"] -->|prompt|    T
///     T            -->|generated| G["🟩 🟩 🟩 🟩 …"]
///
///     style P fill:#ffff,stroke:#ffff
///     style G fill:#ffff,stroke:#ffff
/// ```
///
/// ```
/// use ml/repos/hf::HfHub
/// use ml/repos/hf::fetch
/// use ml/models/mistral::Mistral
/// use ml/models/mistral::load
/// use ml/models/mistral::generate
/// use std/engine/util::startup
///
/// treatment example()
///   model hub:     HfHub(repo_id = "mistralai/Mistral-7B-v0.1")
///   model mistral: Mistral(temperature = 0.7, max_new_tokens = 256)
///   input  prompt:    Stream<string>
///   output generated: Stream<string>
/// {
///     startup()
///     fetch[hub=hub]()
///     load[mistral=mistral]()
///     generate[mistral=mistral]()
///
///     startup.trigger    -> fetch.trigger
///     fetch.safetensors  -> load.safetensors
///     fetch.tokenizer    -> load.tokenizer
///     Self.prompt        -> generate.prompt
///     generate.generated -> Self.generated
/// }
/// ```
#[mel_treatment(
    model mistral Mistral
    input  prompt    Stream<string>
    output generated Stream<string>
)]
pub async fn generate() {
    let model_arc = MistralModel::into(mistral);
    let mistral_struct = model_arc.inner();

    while let Ok(val) = prompt.recv_one().await {
        let text = GetData::<String>::try_data(val).unwrap_or_default();

        #[cfg(feature = "real")]
        {
            if let Some(reply_rx) = mistral_struct.enqueue(text) {
                // recv_async() yields to the executor between tokens;
                // the OS thread runs inference concurrently.
                while let Ok(word) = reply_rx.recv_async().await {
                    check!(generated.send_one(Value::String(word)).await);
                }
            }
        }

        #[cfg(not(feature = "real"))]
        {
            let _ = &text;
        }
    }
}
