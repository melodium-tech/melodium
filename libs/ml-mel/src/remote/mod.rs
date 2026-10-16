pub mod llm;
pub mod stt;
pub mod tts;

#[cfg(feature = "real")]
use ::llm::{builder::LLMBuilder, LLMProvider};
#[cfg(feature = "real")]
use melodium_core::common::executive::{Secret, SecretsAccess};
#[cfg(feature = "real")]
use std::sync::{Arc, Mutex};

/// Gives the provider held in `slot`, building it with `builder` on first use.
///
/// Providers are built when first used rather than when models are initialized,
/// as secrets are only revealed once every model is initialized. The API key is
/// only revealed to be given to the builder.
#[cfg(feature = "real")]
pub(crate) async fn provider(
    slot: &Mutex<Option<Arc<Box<dyn LLMProvider>>>>,
    builder: impl FnOnce() -> Result<LLMBuilder, String>,
    api_key: Option<Secret>,
    secrets_access: SecretsAccess,
) -> Result<Arc<Box<dyn LLMProvider>>, String> {
    if let Some(provider) = slot.lock().unwrap().clone() {
        return Ok(provider);
    }

    let builder = builder()?;
    let provider = match api_key {
        Some(api_key) => secrets_access
            .reveal_str(&api_key, |api_key| builder.api_key(api_key).build())
            .await
            .map_err(|error| error.to_string())?,
        None => builder.build(),
    }
    .map(Arc::new)
    .map_err(|error| format!("failed to build provider: {error}"))?;

    *slot.lock().unwrap() = Some(Arc::clone(&provider));
    Ok(provider)
}
