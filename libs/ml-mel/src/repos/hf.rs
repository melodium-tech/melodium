use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
use std::sync::Weak;

/// HuggingFace Hub repository configuration.
///
/// Holds the connection and repository parameters needed to reach a repository on the
/// HuggingFace Hub.  Does no work by itself — use the `fetch` treatment to trigger the
/// actual resolution and download of files.
///
/// - `repo_id`: repository identifier, e.g. `"mistralai/Mistral-7B-v0.1"`.
/// - `repo_type`: `"model"` (default), `"dataset"`, or `"space"`.
/// - `revision`: git revision to pin — branch, tag, or commit hash (default `"main"`).
/// - `endpoint`: Hub API base URL (default `"https://huggingface.co"`; override for mirrors).
/// - `cache_dir`: local cache directory (empty string uses the default `~/.cache/huggingface/hub`).
/// - `token`: HuggingFace access token for private repositories (empty string means no token).
///
/// ℹ️ Use `HfHub` together with `fetch` — `HfHub` holds the configuration while `fetch`
/// performs the actual network and cache operations when triggered.
///
/// ```
/// use ml/repos/hf::HfHub
/// use ml/repos/hf::fetch
/// use ml/models/mistral::Mistral
/// use ml/models/mistral::load
/// use std/engine/util::startup
///
/// treatment example()
///   model hub:    HfHub(repo_id = "mistralai/Mistral-7B-v0.1")
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
#[mel_model(
    param repo_id   string  none
    param repo_type string  "model"
    param revision  string  "main"
    param endpoint  string  "https://huggingface.co"
    param cache_dir string  ""
    param token     string  ""
)]
#[derive(Debug)]
pub struct HfHub {
    #[allow(unused)]
    model: Weak<HfHubModel>,
}

impl HfHub {
    fn new(model: Weak<HfHubModel>) -> Self {
        Self { model }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}

/// Resolve and download files from a HuggingFace Hub repository.
///
/// On `trigger`, contacts the Hub API (or the local cache if files are already present),
/// lists all `.safetensors` shards and `tokenizer.json` in the configured repository,
/// downloads any files not already cached, then emits their local filesystem paths.
///
/// `safetensors` emits one path per shard in sorted order — this covers both single-file
/// and multi-shard models transparently.  `tokenizer` emits the single path to
/// `tokenizer.json`.  If any network or cache error occurs, `failed` and `error` are
/// emitted instead and `safetensors` / `tokenizer` remain empty.
///
/// ℹ️ Wire `safetensors` and `tokenizer` directly into a `load` treatment to initialise a
/// `Mistral` model.
///
/// ⚠️ The Hub API calls are synchronous and may block for several minutes on the first
/// run while shards are downloaded.  Subsequent runs return cached paths immediately.
///
/// ```mermaid
/// graph LR
///     T("fetch()")
///     B["〈🟦〉"]           -->|trigger|     T
///     T -->|safetensors|   S["🟩 🟩 🟩 …"]
///     T -->|tokenizer|     K["〈🟨〉"]
///     T -->|failed|        F["〈🟦〉"]
///     T -->|error|         E["〈🟨〉"]
///
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
///     style K fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
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
///   model mistral: Mistral()
///   input  prompt:    Stream<string>
///   output generated: Stream<string>
/// {
///     startup()
///     fetch[hub=hub]()
///     load[mistral=mistral]()
///     generate[mistral=mistral]()
///
///     startup.trigger   -> fetch.trigger
///     fetch.safetensors -> load.safetensors
///     fetch.tokenizer   -> load.tokenizer
///     load.loaded       -> generate.ready
///     Self.prompt       -> generate.prompt
///     generate.generated -> Self.generated
/// }
/// ```
#[mel_treatment(
    model hub        HfHub
    input  trigger     Block<void>
    output safetensors Stream<string>
    output tokenizer   Block<string>
    output failed      Block<void>
    output error       Block<string>
)]
pub async fn fetch() {
    if trigger.recv_one().await.is_err() {
        return;
    }

    let model_arc = HfHubModel::into(hub);
    let _hub = model_arc.inner();

    #[cfg(feature = "real")]
    {
        use hf_hub::{
            api::sync::{ApiBuilder, ApiError},
            Repo, RepoType,
        };

        let repo_id = model_arc.get_repo_id();
        let repo_type = model_arc.get_repo_type();
        let revision = model_arc.get_revision();
        let endpoint = model_arc.get_endpoint();
        let cache_dir = model_arc.get_cache_dir();
        let token_str = model_arc.get_token();

        // Hub calls are blocking I/O — run off the async executor.
        let result =
            async_std::task::spawn_blocking(move || -> Result<(Vec<String>, String), String> {
                let repo_type = match repo_type.as_str() {
                    "dataset" => RepoType::Dataset,
                    "space" => RepoType::Space,
                    _ => RepoType::Model,
                };

                let mut builder =
                    ApiBuilder::new()
                        .with_endpoint(endpoint)
                        .with_token(if token_str.is_empty() {
                            None
                        } else {
                            Some(token_str)
                        });

                if !cache_dir.is_empty() {
                    builder = builder.with_cache_dir(std::path::PathBuf::from(cache_dir));
                }

                let api = builder.build().map_err(|e| e.to_string())?;
                let repo = api.repo(Repo::with_revision(repo_id, repo_type, revision));
                let info = repo.info().map_err(|e: ApiError| e.to_string())?;

                let mut shard_paths: Vec<String> = info
                    .siblings
                    .iter()
                    .filter(|s| s.rfilename.ends_with(".safetensors"))
                    .map(|s| {
                        repo.get(&s.rfilename)
                            .map(|p| p.to_string_lossy().into_owned())
                            .map_err(|e: ApiError| e.to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                shard_paths.sort();

                let tokenizer_path = repo
                    .get("tokenizer.json")
                    .map(|p| p.to_string_lossy().into_owned())
                    .map_err(|e: ApiError| e.to_string())?;

                Ok((shard_paths, tokenizer_path))
            })
            .await;

        match result {
            Ok((shard_paths, tokenizer_path)) => {
                for path in shard_paths {
                    check!(safetensors.send_one(Value::String(path)).await);
                }
                let _ = tokenizer.send_one(Value::String(tokenizer_path)).await;
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(Value::String(err)).await;
            }
        }
    }

    #[cfg(not(feature = "real"))]
    {
        let _ = &_hub;
    }
}
