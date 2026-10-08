//! Secrets received at runtime, added to environments.
//!
//! These treatments do what `|with_secret_variables` and `|with_secret_stdin` do, for
//! secrets arriving on inputs, such as those given by `vault::get`.

use super::*;
use std::sync::Arc;

/// Adds secret variables received at runtime to `base`.
///
/// Same as `process/environment::|with_secret_variables`, for secrets received at runtime:
/// each entry of `secret_variables`, a map built with the treatments of
/// `std/data/map/block`, gives the `Secret<string>` value of a variable, possibly in an
/// option. Values are revealed only when a command is run, and never put in command
/// arguments. Running a command fails if an entry holds no `Secret<string>`.
///
/// Commands take the environment as an option, see `std/ops/option/block::wrap`.
#[mel_treatment(
    input base Block<Environment>
    input secret_variables Block<Map>
    output environment Block<Environment>
)]
pub async fn with_secret_variables() {
    if let (Ok(base), Ok(variables)) = (
        base.recv_one_as::<Arc<Environment>>().await,
        secret_variables.recv_one_as::<Arc<Map>>().await,
    ) {
        let mut updated = Arc::unwrap_or_clone(base);
        add_secret_variables(&mut updated, Arc::unwrap_or_clone(variables).map);
        let _ = environment
            .send_one_as(Arc::new(updated) as Arc<dyn Data>)
            .await;
    }
}

/// Writes a secret received at runtime to the standard input of commands run with `base`,
/// before anything else.
///
/// Same as `process/environment::|with_secret_stdin`. The secret is revealed only when
/// a command is run.
///
/// Commands take the environment as an option, see `std/ops/option/block::wrap`.
#[mel_treatment(
    input base Block<Environment>
    input secret Block<Secret<string>>
    output environment Block<Environment>
)]
pub async fn with_secret_stdin() {
    if let (Ok(base), Ok(Value::Secret(received))) = (
        base.recv_one_as::<Arc<Environment>>().await,
        secret.recv_one().await,
    ) {
        let mut updated = Arc::unwrap_or_clone(base);
        updated.secret_stdin = Some(received);
        let _ = environment
            .send_one_as(Arc::new(updated) as Arc<dyn Data>)
            .await;
    }
}
