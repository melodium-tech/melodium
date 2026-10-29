//! Transfers with object storage, through requests presigned by the API.
//!
//! Presigned URLs are credentials: errors describe what failed without them, and what
//! storage answers is reduced to its status and error code.

use super::{PresignedRequest, Upload, UploadedPart};
use async_compat::Compat;
use futures::{AsyncSeekExt, AsyncWriteExt};
use reqwest::header::{HeaderName, HeaderValue, CONTENT_LENGTH, ETAG, RANGE};
use reqwest::{Method, RequestBuilder, StatusCode};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

/// Attempts made for each request, the first one included.
const ATTEMPTS: u32 = 4;
/// Wait before the second attempt, doubled for each following one.
const FIRST_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Why a request failed.
enum Failure {
    /// Worth trying again: storage unreachable, interrupted, or busy.
    Transient(String),
    /// Refused for good.
    Refused(String),
}

/// Uploads the first `size` bytes of `file` as `upload` tells, giving the sha256 of what was
/// sent, and the parts uploaded for a multipart upload.
///
/// Each part is read once, hashed, and sent again as it is when an attempt fails.
pub(crate) async fn upload(
    file: Arc<Mutex<File>>,
    size: u64,
    upload: &Upload,
) -> Result<(String, Vec<UploadedPart>), String> {
    let mut hasher = Sha256::new();
    let mut uploaded = Vec::new();
    match upload {
        Upload::Single { request } => {
            let data = read_part(&file, 0, size).await?;
            hasher.update(&data);
            put(request, data).await?;
        }
        Upload::Multipart { part_size, parts } => {
            if *part_size == 0 {
                return Err("Cadence.CI gave parts of no size to upload".to_string());
            }
            let count = size.div_ceil(*part_size).max(1);
            if parts.len() as u64 != count {
                return Err(format!(
                    "Cadence.CI gave {} parts to upload {size} bytes in parts of {part_size}",
                    parts.len()
                ));
            }
            for (index, request) in parts.iter().enumerate() {
                let offset = index as u64 * part_size;
                let data = read_part(&file, offset, (*part_size).min(size - offset)).await?;
                hasher.update(&data);
                let number = index as u32 + 1;
                let etag = put(request, data)
                    .await?
                    .ok_or_else(|| format!("storage gave no entity tag for part {number}"))?;
                uploaded.push(UploadedPart {
                    part_number: number,
                    etag,
                });
            }
        }
    }
    Ok((hex::encode(hasher.finalize()), uploaded))
}

/// Downloads the object `request` gives into `file`, from its start, checking that it has
/// `size` bytes and the `sha256` given.
///
/// An interrupted download resumes where it stopped, with a range request.
pub(crate) async fn download(
    request: &PresignedRequest,
    size: u64,
    sha256: &str,
    file: File,
) -> Result<File, String> {
    let mut file = smol::Unblock::new(file);
    let mut hasher = Sha256::new();
    let mut written = 0u64;
    let mut failure = String::new();

    for attempt in 0..ATTEMPTS {
        if attempt > 0 {
            smol::Timer::after(FIRST_RETRY_DELAY * 2u32.pow(attempt - 1)).await;
        }
        let mut builder = build(request)?;
        if written > 0 {
            builder = builder.header(RANGE, format!("bytes={written}-"));
        }

        // The HTTP client runs on Tokio.
        let outcome = Compat::new(async {
            let mut response = builder
                .send()
                .await
                .map_err(|err| Failure::Transient(unreachable(err, request)))?;
            let status = response.status();
            if !status.is_success() {
                let body = response.bytes().await.unwrap_or_default();
                return Err(refused(status, &body));
            }
            if written > 0 && status != StatusCode::PARTIAL_CONTENT {
                // Storage gives the whole object again.
                written = 0;
                hasher = Sha256::new();
                file.seek(SeekFrom::Start(0)).await.map_err(local)?;
                file.with_mut(|file| file.set_len(0)).await.map_err(local)?;
            }
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|err| Failure::Transient(unreachable(err, request)))?
            {
                written += chunk.len() as u64;
                if written > size {
                    return Err(Failure::Refused(format!(
                        "storage gave more than the {size} bytes announced"
                    )));
                }
                hasher.update(&chunk);
                file.write_all(&chunk).await.map_err(local)?;
            }
            Ok(())
        })
        .await;

        match outcome {
            Ok(()) if written == size => {
                failure.clear();
                break;
            }
            Ok(()) => failure = format!("storage gave {written} bytes of {size}"),
            Err(Failure::Transient(problem)) => failure = problem,
            Err(Failure::Refused(problem)) => return Err(problem),
        }
    }
    if !failure.is_empty() {
        return Err(failure);
    }

    if !hex::encode(hasher.finalize()).eq_ignore_ascii_case(sha256) {
        return Err("what storage gave does not match its sha256".to_string());
    }
    file.flush()
        .await
        .map_err(|err| format!("cannot be written: {err}"))?;
    Ok(file.into_inner().await)
}

/// Reads `length` bytes of `file` from `offset`.
async fn read_part(file: &Arc<Mutex<File>>, offset: u64, length: u64) -> Result<Vec<u8>, String> {
    let file = Arc::clone(file);
    smol::unblock(move || {
        let mut file = file.lock().unwrap_or_else(PoisonError::into_inner);
        let mut data = vec![0; length as usize];
        file.seek(SeekFrom::Start(offset))
            .and_then(|_| file.read_exact(&mut data))
            .map(|()| data)
            .map_err(|err| format!("cannot be read while uploading it: {err}"))
    })
    .await
}

/// Sends `data` with `request`, giving the entity tag storage answers.
async fn put(request: &PresignedRequest, data: Vec<u8>) -> Result<Option<String>, String> {
    let body = bytes::Bytes::from(data);
    let mut failure = String::new();
    for attempt in 0..ATTEMPTS {
        if attempt > 0 {
            smol::Timer::after(FIRST_RETRY_DELAY * 2u32.pow(attempt - 1)).await;
        }
        let builder = build(request)?
            .header(CONTENT_LENGTH, body.len())
            .body(body.clone());

        // The HTTP client runs on Tokio.
        let outcome = Compat::new(async {
            let response = builder
                .send()
                .await
                .map_err(|err| Failure::Transient(unreachable(err, request)))?;
            let status = response.status();
            let etag = response
                .headers()
                .get(ETAG)
                .and_then(|etag| etag.to_str().ok())
                .map(str::to_string);
            let answer = response
                .bytes()
                .await
                .map_err(|err| Failure::Transient(unreachable(err, request)))?;
            if status.is_success() {
                Ok(etag)
            } else {
                Err(refused(status, &answer))
            }
        })
        .await;

        match outcome {
            Ok(etag) => return Ok(etag),
            Err(Failure::Transient(problem)) => failure = problem,
            Err(Failure::Refused(problem)) => return Err(problem),
        }
    }
    Err(failure)
}

/// Builds the request presigned as `request`.
fn build(request: &PresignedRequest) -> Result<RequestBuilder, String> {
    let method = Method::from_bytes(request.method.as_bytes())
        .map_err(|_| format!("Cadence.CI gave an invalid method '{}'", request.method))?;
    let mut builder = crate::http::CLIENT
        .as_ref()
        .map_err(Clone::clone)?
        .request(method, &request.url);
    for (name, value) in &request.headers {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| format!("Cadence.CI gave an invalid header name '{name}'"))?;
        let value = HeaderValue::from_str(value)
            .map_err(|_| format!("Cadence.CI gave an invalid value for header '{name}'"))?;
        builder = builder.header(name, value);
    }
    Ok(builder)
}

/// Describes why storage could not be reached for `request`, without its URL.
fn unreachable(err: reqwest::Error, request: &PresignedRequest) -> String {
    let err = err.without_url();
    let mut description = format!("storage cannot be reached: {err}");
    let mut source = std::error::Error::source(&err);
    while let Some(cause) = source {
        description.push_str(": ");
        description.push_str(&cause.to_string());
        source = cause.source();
    }
    redact(description, &request.url)
}

/// Describes a refusal of storage, with its status and the error code it gives, if any.
///
/// The rest of what storage answers is left out, as it can repeat the signature.
fn refused(status: StatusCode, body: &[u8]) -> Failure {
    let body = String::from_utf8_lossy(body);
    let code = body
        .split_once("<Code>")
        .and_then(|(_, rest)| rest.split_once("</Code>"))
        .map(|(code, _)| code)
        .filter(|code| {
            code.len() <= 64 && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '.')
        });
    let description = match code {
        Some(code) => format!("storage answered {status}: {code}"),
        None => format!("storage answered {status}"),
    };
    if status.is_server_error()
        || status == StatusCode::TOO_MANY_REQUESTS
        || status == StatusCode::REQUEST_TIMEOUT
    {
        Failure::Transient(description)
    } else {
        Failure::Refused(description)
    }
}

/// Fails for good on a local file error.
fn local(err: std::io::Error) -> Failure {
    Failure::Refused(format!("cannot be written: {err}"))
}

/// Removes every trace of `url` and of its query, holding the signature, from `description`.
fn redact(description: String, url: &str) -> String {
    let mut description = description.replace(url, "<presigned URL>");
    if let Some((_, query)) = url.split_once('?') {
        if !query.is_empty() {
            description = description.replace(query, "<signature>");
        }
    }
    description
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refusals_keep_only_the_error_code() {
        let body = b"<?xml version=\"1.0\"?><Error><Code>SignatureDoesNotMatch</Code><SignatureProvided>abcdef0123</SignatureProvided></Error>";
        match refused(StatusCode::FORBIDDEN, body) {
            Failure::Refused(description) => {
                assert_eq!(
                    description,
                    "storage answered 403 Forbidden: SignatureDoesNotMatch"
                );
            }
            Failure::Transient(_) => panic!("a refusal is final"),
        }
        assert!(matches!(
            refused(StatusCode::SERVICE_UNAVAILABLE, b""),
            Failure::Transient(_)
        ));
    }

    #[test]
    fn descriptions_lose_presigned_urls() {
        let url = "https://bucket.example/orga/caches/1?X-Amz-Signature=abcdef0123";
        assert_eq!(
            redact(format!("failed for {url}"), url),
            "failed for <presigned URL>"
        );
        assert_eq!(
            redact("failed with X-Amz-Signature=abcdef0123".to_string(), url),
            "failed with <signature>"
        );
    }
}
