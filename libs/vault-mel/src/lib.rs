#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

#[cfg(feature = "real")]
mod client;

use async_trait::async_trait;
use melodium_core::common::descriptor::DataType;
use melodium_core::common::executive::{
    Secret as ExecutiveSecret, SecretOrigin, SecretPolicy, SecretSource, SecretsAccess,
};
use melodium_core::*;
use melodium_macro::{mel_model, mel_package, mel_treatment};
use std::sync::{Arc, Weak};

/// HashiCorp Vault or OpenBao server, providing secrets.
///
/// The model registers a secret source named `source`, so that secrets located as
/// `vault:<path>#<field>` are read from the server when revealed, such as
/// `vault:kv/data/app/db#password` for the `password` field of the `app/db` key/value
/// secret in a `kv` engine. The field can be omitted if the secret has only one.
///
/// - `address`: address of the server, such as `https://vault.example.com:8200`.
/// - `source`: name of the source, the scheme of locators (`vault` by default); two models
/// with the same source, or a source named `env` or `file`, make the launch fail.
/// - `namespace`: namespace to work in, if any.
///
/// Authentication is chosen by `auth`:
/// - `token`: uses `token` as client token.
/// - `approle`: logs in with `role_id` and `secret_id`.
/// - `kubernetes`: logs in as `role` with the service account token `kubernetes_token`.
/// - `jwt`: logs in as `role` with the `jwt` token, such as a GitLab CI ID token.
/// - `github`: logs in as `role` with a GitHub Actions OIDC token, requested for
/// the `github_audience` audience using `github_token`.
///
/// `auth_mount` is the path where the authentication method is enabled, by default
/// `approle`, `kubernetes` or `jwt` (also for `github`).
///
/// Credentials are secrets, by default read from the usual environment variables,
/// prefixed with `MELODIUM_SECRET_` as programs only get those
/// (`MELODIUM_SECRET_VAULT_TOKEN`, `MELODIUM_SECRET_VAULT_ROLE_ID`,
/// `MELODIUM_SECRET_VAULT_SECRET_ID`, `MELODIUM_SECRET_VAULT_ID_TOKEN`,
/// `MELODIUM_SECRET_ACTIONS_ID_TOKEN_REQUEST_TOKEN`), and from the Kubernetes service
/// account token file.
/// The model reveals them as `vault::Vault`, so they can be restricted to it
/// with `std/secret::|reveal_only_by`.
///
/// Read secrets are kept for `cache_ttl` seconds (`0` disables caching).
/// `prefetch` lists the paths of secrets to read at startup, separated by spaces.
/// Running with `--check-secrets` also reads the secrets given as parameters at startup.
#[mel_model(
    param address string none
    param source string "vault"
    param namespace string ""
    param auth string "token"
    param auth_mount string ""
    param role string ""
    param token Secret<string> "env:MELODIUM_SECRET_VAULT_TOKEN"
    param role_id Secret<string> "env:MELODIUM_SECRET_VAULT_ROLE_ID"
    param secret_id Secret<string> "env:MELODIUM_SECRET_VAULT_SECRET_ID"
    param jwt Secret<string> "env:MELODIUM_SECRET_VAULT_ID_TOKEN"
    param kubernetes_token Secret<string> "file:/var/run/secrets/kubernetes.io/serviceaccount/token"
    param github_token Secret<string> "env:MELODIUM_SECRET_ACTIONS_ID_TOKEN_REQUEST_TOKEN"
    param github_audience string ""
    param cache_ttl u64 300
    param prefetch string ""
    secrets_access
    secret_sources register_source
    continuous (prefetch)
    shutdown shutdown
)]
#[derive(Debug)]
pub struct Vault {
    model: Weak<VaultModel>,
    #[cfg(feature = "real")]
    client: Arc<client::Client>,
}

impl Vault {
    fn new(model: Weak<VaultModel>) -> Self {
        Self {
            #[cfg(feature = "real")]
            client: {
                let client = Arc::new(client::Client::new(model.clone()));
                melodium_core::common::executive::register_wipe(Arc::downgrade(&client)
                    as std::sync::Weak<dyn melodium_core::common::executive::Wipe>);
                client
            },
            model,
        }
    }

    fn register_source(&self, access: &SecretsAccess) {
        // A scheme registered twice makes the launch fail.
        let _ = access.register_source(
            &self.model.upgrade().unwrap().get_source(),
            Arc::new(VaultSource {
                model: self.model.clone(),
            }),
        );
    }

    async fn prefetch(&self) {
        #[cfg(feature = "real")]
        if let Some(model) = self.model.upgrade() {
            for path in model.get_prefetch().split_whitespace() {
                if let Err(err) = self.client.read(path).await {
                    model
                        .world()
                        .log(
                            melodium_core::common::executive::Level::Error,
                            "vault".to_string(),
                            format!("cannot prefetch '{path}': {err}"),
                            None,
                        )
                        .await;
                }
            }
        }
    }

    fn shutdown(&self) {
        #[cfg(feature = "real")]
        async_std::task::block_on(self.client.forget());
    }

    /// Gives the value at `path` (`<path>#<field>`) as `datatype`.
    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
        #[cfg(feature = "real")]
        {
            self.client.resolve(path, datatype).await
        }
        #[cfg(feature = "mock")]
        {
            let _ = (path, datatype);
            Err("vault is not available in this build".to_string())
        }
    }
}

/// Secret source reading from the `Vault` model it belongs to.
#[derive(Debug)]
struct VaultSource {
    model: Weak<VaultModel>,
}

#[async_trait]
impl SecretSource for VaultSource {
    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
        match self.model.upgrade() {
            Some(model) => model.inner().resolve(path, datatype).await,
            None => Err("vault model has ended".to_string()),
        }
    }
}

/// Gives a secret stored at `path` in vault.
///
/// `path` is `<path>#<field>`, such as `kv/data/app/db#password`, read from the source
/// of `vault`. The secret is named `name` and gets the default policy.
/// It is read once to check it exists, and resolved again when revealed.
///
/// If the secret cannot be read, `failed` is emitted and `error` contains the reason.
#[mel_treatment(
    model vault Vault
    input path Block<string>
    output secret Block<Secret<string>>
    output failed Block<void>
    output error Block<string>
)]
pub async fn get(name: string) {
    let model = VaultModel::into(vault);
    if let Ok(path) = path.recv_one_as::<String>().await {
        let checked = match ExecutiveSecret::new(
            name,
            DataType::String,
            SecretPolicy::default(),
            SecretOrigin::Locator(format!("{}:{path}", model.get_source())),
        ) {
            Ok(created) => model
                .secrets_access()
                .check_resolution(&created)
                .await
                .map(|_| created),
            Err(err) => Err(err),
        };
        match checked {
            Ok(checked) => {
                let _ = secret.send_one(Value::Secret(checked)).await;
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(err.to_string().into()).await;
            }
        }
    }
}

mel_package!();
