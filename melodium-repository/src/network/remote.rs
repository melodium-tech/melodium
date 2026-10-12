use std::path::PathBuf;

use super::NetworkRepositoryConfiguration;
use crate::global::Package as PackageDetails;
use crate::technical::{Element, Package};
use crate::{RepositoryError, RepositoryResult};
use once_cell::sync::OnceCell;
use smol::{block_on, io::AsyncWriteExt};
use trillium_client::{Client, KnownHeaderName, Url};

pub fn get_tech_packages(
    config: &NetworkRepositoryConfiguration,
) -> RepositoryResult<Vec<Package>> {
    let url = Url::parse(&format!("{}/packages.json", config.base_url)).unwrap();

    block_on(async {
        let mut response = http_client()
            .get(url)
            .with_request_header(KnownHeaderName::UserAgent, config.user_agent.clone())
            .await
            .map_err(|err| RepositoryError::network_error(16, err.to_string()))?;

        if is_success(&response) {
            let body = response
                .response_body()
                .read_bytes()
                .await
                .map_err(|err| RepositoryError::network_error(17, err.to_string()))?;

            serde_json::from_slice(&body).map_err(|err| RepositoryError::json_error(15, err))
        } else {
            Err(RepositoryError::network_error(46, reason(&response)))
        }
    })
}

pub fn get_detail_package(
    config: &NetworkRepositoryConfiguration,
    package: &Package,
) -> RepositoryResult<PackageDetails> {
    let url = Url::parse(&format!(
        "{}/{}/package.json",
        config.base_url,
        package.get_path().to_string_lossy(),
    ))
    .unwrap();

    block_on(async {
        let mut response = http_client()
            .get(url)
            .with_request_header(KnownHeaderName::UserAgent, config.user_agent.clone())
            .await
            .map_err(|err| RepositoryError::network_error(41, err.to_string()))?;
        if is_success(&response) {
            let body = response
                .response_body()
                .read_bytes()
                .await
                .map_err(|err| RepositoryError::network_error(42, err.to_string()))?;
            serde_json::from_slice(&body).map_err(|err| RepositoryError::json_error(43, err))
        } else {
            Err(RepositoryError::network_error(47, reason(&response)))
        }
    })
}

pub fn get_element_package(
    config: &NetworkRepositoryConfiguration,
    _element: Element,
    url_path: PathBuf,
    file_path: PathBuf,
) -> RepositoryResult<()> {
    let url = Url::parse(&format!(
        "{}/{}",
        config.base_url,
        url_path.to_string_lossy(),
    ))
    .unwrap();

    block_on(async {
        smol::fs::create_dir_all(file_path.parent().unwrap_or(&PathBuf::default()))
            .await
            .map_err(|err| RepositoryError::fs_error(49, err))?;

        let mut file = smol::fs::File::create(file_path)
            .await
            .map_err(|err| RepositoryError::fs_error(34, err))?;

        let mut response = http_client()
            .get(url)
            .with_request_header(KnownHeaderName::UserAgent, config.user_agent.clone())
            .await
            .map_err(|err| RepositoryError::network_error(33, err.to_string()))?;

        if is_success(&response) {
            smol::io::copy(response.response_body(), &mut file)
                .await
                .map_err(|err| RepositoryError::fs_error(35, err))?;
            file.flush()
                .await
                .map_err(|err| RepositoryError::fs_error(35, err))?;
            Ok(())
        } else {
            Err(RepositoryError::network_error(48, reason(&response)))
        }
    })
}

fn is_success(response: &trillium_client::Conn) -> bool {
    response
        .status()
        .map(|status| status.is_success())
        .unwrap_or(false)
}

fn reason(response: &trillium_client::Conn) -> String {
    response
        .status()
        .map(|status| status.canonical_reason())
        .unwrap_or("no response")
        .to_string()
}

fn http_client() -> &'static Client {
    static CLIENT: OnceCell<Client> = OnceCell::new();
    CLIENT.get_or_init(|| {
        Client::new(trillium_rustls::RustlsConfig::<trillium_smol::ClientConfig>::default())
            .with_default_pool()
    })
}
