//! The run an engine is part of on api.melodium.tech, as programs reach it.

use melodium_core::*;
use melodium_macro::mel_function;

/// Return the address of the run on api.melodium.tech, to call its endpoints.
///
/// It is `<MELODIUM_API_URL>/execution/run/<MELODIUM_RUN_ID>`, as api.melodium.tech workers are
/// configured, such as `https://api.melodium.tech/0.1/execution/run/<id>`, and none elsewhere.
/// Requests to it carry the token of the run, `api-run:token`, in their secret headers.
///
/// The address is not a credential.
#[mel_function]
pub fn api_url() -> Option<string> {
    crate::secret::Api::from_environment().ok().map(|api| {
        format!(
            "{}/execution/run/{}",
            api.url.trim_end_matches('/'),
            api.run_id
        )
    })
}

/// Return the address of api.melodium.tech, as the run reaches it.
///
/// It is `MELODIUM_API_URL`, as api.melodium.tech workers are configured, such as
/// `https://api.melodium.tech/0.1`, and none elsewhere. Requests to it carry the token of the
/// run, `api-run:token`, in their secret headers, such as to read artifacts at their stable
/// addresses (`<address>/artifacts/<run>/<name>`).
///
/// The address is not a credential.
#[mel_function]
pub fn api_root_url() -> Option<string> {
    crate::secret::Api::from_environment()
        .ok()
        .map(|api| api.url.trim_end_matches('/').to_string())
}
