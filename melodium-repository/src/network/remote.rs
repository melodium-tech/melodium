use std::path::PathBuf;

use super::NetworkRepositoryConfiguration;
use crate::global::Package as PackageDetails;
use crate::technical::{Element, Package};
use crate::{RepositoryError, RepositoryResult};
use async_std::task::block_on;
use http_client::{http_types::Url, HttpClient, Request};
use once_cell::sync::OnceCell;

pub fn get_tech_packages(
    config: &NetworkRepositoryConfiguration,
) -> RepositoryResult<Vec<Package>> {
    let mut request =
        Request::get(Url::parse(&format!("{}/packages.json", config.base_url)).unwrap());
    request.insert_header("User-Agent", config.user_agent.clone());

    block_on(async {
        let mut response = http_client()
            .send(request)
            .await
            .map_err(|err| RepositoryError::network_error(16, err.to_string()))?;

        if response.status().is_success() {
            let body = response
                .body_string()
                .await
                .map_err(|err| RepositoryError::network_error(17, err.to_string()))?;

            serde_json::from_str(&body).map_err(|err| RepositoryError::json_error(15, err))
        } else {
            Err(RepositoryError::network_error(
                46,
                response.status().canonical_reason().to_string(),
            ))
        }
    })
}

pub fn get_detail_package(
    config: &NetworkRepositoryConfiguration,
    package: &Package,
) -> RepositoryResult<PackageDetails> {
    let mut request = Request::get(
        Url::parse(&format!(
            "{}/{}/package.json",
            config.base_url,
            package.get_path().to_string_lossy(),
        ))
        .unwrap(),
    );
    request.insert_header("User-Agent", config.user_agent.clone());

    block_on(async {
        let mut response = http_client()
            .send(request)
            .await
            .map_err(|err| RepositoryError::network_error(41, err.to_string()))?;
        if response.status().is_success() {
            let body = response
                .body_string()
                .await
                .map_err(|err| RepositoryError::network_error(42, err.to_string()))?;
            serde_json::from_str(&body).map_err(|err| RepositoryError::json_error(43, err))
        } else {
            Err(RepositoryError::network_error(
                47,
                response.status().canonical_reason().to_string(),
            ))
        }
    })
}

pub fn get_element_package(
    config: &NetworkRepositoryConfiguration,
    _element: Element,
    url_path: PathBuf,
    file_path: PathBuf,
) -> RepositoryResult<()> {
    let mut request = Request::get(
        Url::parse(&format!(
            "{}/{}",
            config.base_url,
            url_path.to_string_lossy(),
        ))
        .unwrap(),
    );
    request.insert_header("User-Agent", config.user_agent.clone());

    block_on(async {
        async_std::fs::create_dir_all(file_path.parent().unwrap_or(&PathBuf::default()))
            .await
            .map_err(|err| RepositoryError::fs_error(49, err))?;

        let mut file = async_std::fs::File::create(file_path)
            .await
            .map_err(|err| RepositoryError::fs_error(34, err))?;

        let mut response = http_client()
            .send(request)
            .await
            .map_err(|err| RepositoryError::network_error(33, err.to_string()))?;

        if response.status().is_success() {
            async_std::io::copy(&mut response, &mut file)
                .await
                .map_err(|err| RepositoryError::fs_error(35, err))?;
            Ok(())
        } else {
            Err(RepositoryError::network_error(
                48,
                response.status().canonical_reason().to_string(),
            ))
        }
    })
}

fn http_client() -> &'static dyn HttpClient {
    static CLIENT: OnceCell<http_client::h1::H1Client> = OnceCell::new();
    CLIENT.get_or_init(|| http_client::h1::H1Client::new())
}
