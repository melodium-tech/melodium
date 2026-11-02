//! Transfers through prepared requests, made as given with a client of their own: downloads
//! resumed after a cut.
//!
//! Prepared URLs often are credentials, such as presigned ones: they never appear in errors,
//! which give the status of answers and the `<Code>` of XML error bodies instead.

use crate::client::PreparedRequest;
use core::future::Future;
use core::time::Duration;
use futures::AsyncReadExt;
use trillium::{HeaderName, HeaderValue, Method};
use trillium_client::{Client, Conn, Url};

/// Size of the chunks read from answers.
const CHUNK: usize = 1 << 20;
/// Longest wait between two attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(16);
/// Longest part of an error body read to find its `<Code>`.
const MAX_ERROR_BODY: usize = 4096;

/// Client making prepared requests, without the default headers of any `HttpClient` model.
#[cfg(feature = "real")]
pub(crate) fn client() -> &'static Client {
    static CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::new(
            trillium_rustls::RustlsConfig::default()
                .with_tcp_config(trillium_smol::ClientConfig::new().with_nodelay(true)),
        )
        .with_default_pool()
        .with_default_header(
            trillium::KnownHeaderName::UserAgent,
            crate::client::USER_AGENT,
        )
    })
}

/// Whether an answer of `status` is worth another attempt.
pub(crate) fn retriable(status: u16) -> bool {
    status >= 500 || status == 429 || status == 408
}

/// Wait before the attempt following `attempt` failed ones.
pub(crate) fn backoff(attempt: u32) -> Duration {
    Duration::from_millis(500)
        .saturating_mul(1 << attempt.min(8))
        .min(MAX_BACKOFF)
}

/// Waits before the attempt following `attempt` failed ones.
pub(crate) async fn wait(attempt: u32) {
    #[cfg(feature = "real")]
    smol::Timer::after(backoff(attempt)).await;
    #[cfg(feature = "mock")]
    let _ = attempt;
}

/// Describes a refusal from its status and the start of its body, giving the `<Code>` of XML
/// error bodies, as object storage answers, and nothing else of the body.
pub(crate) fn refusal(status: u16, body: &[u8]) -> String {
    let body = String::from_utf8_lossy(body);
    let code = body
        .split_once("<Code>")
        .and_then(|(_, rest)| rest.split_once("</Code>"))
        .map(|(code, _)| code.trim())
        .filter(|code| {
            !code.is_empty()
                && code.len() <= 64
                && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '.')
        });
    match code {
        Some(code) => format!("storage answered {status} ({code})"),
        None => format!("storage answered {status}"),
    }
}

/// Removes `url`, and its parts holding signatures, from `message`.
pub(crate) fn redact(message: String, url: &Url) -> String {
    let mut message = message.replace(url.as_str(), "<prepared URL>");
    if let Some(query) = url.query().filter(|query| !query.is_empty()) {
        message = message.replace(query, "<query>");
    }
    message
}

/// Builds the connection making `request`, with `extra` headers.
pub(crate) fn connection(
    client: &Client,
    request: &PreparedRequest,
    extra: &[(&'static str, String)],
) -> Result<(Conn, Url), String> {
    let url = Url::parse(&request.url).map_err(|_| "the prepared URL is not valid".to_string())?;
    let method = Method::try_from(request.method.as_str())
        .map_err(|_| format!("'{}' is not an HTTP method", request.method))?;
    let mut conn = client.build_conn(method, url.clone());
    for (name, value) in request
        .headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.clone()))
        .chain(extra.iter().map(|(name, value)| (*name, value.clone())))
    {
        let header_name = HeaderName::from(name.to_string());
        let header_value = HeaderValue::from(value);
        if !header_name.is_valid() || !header_value.is_valid() {
            return Err(format!(
                "header '{name}' of the prepared request is not valid"
            ));
        }
        conn.request_headers_mut()
            .insert(header_name.to_owned(), header_value);
    }
    Ok((conn, url))
}

/// Reads at most `MAX_ERROR_BODY` bytes of the body of `conn`.
pub(crate) async fn error_body(conn: &mut Conn) -> Vec<u8> {
    let mut body = Vec::new();
    let _ = conn
        .response_body()
        .take(MAX_ERROR_BODY as u64)
        .read_to_end(&mut body)
        .await;
    body
}

/// Header `name` of the answer of `conn`, if any.
pub(crate) fn header(conn: &Conn, name: &'static str) -> Option<String> {
    conn.response_headers()
        .get_str(name)
        .map(|value| value.to_string())
}

/// How an attempt of a download ended.
enum Attempt {
    /// Cut, to try again from what was given.
    Cut(String),
    /// Failed for good.
    Failed(String),
}

/// Downloads the object of `request`, giving its bytes through `give`, resuming after a cut
/// up to `retries` times, never giving a byte twice.
///
/// `give` tells whether the bytes were taken: once they are not, the download stops, and
/// `Ok(false)` is given.
pub(crate) async fn download<F, Fut>(
    client: &Client,
    request: &PreparedRequest,
    retries: u32,
    mut give: F,
) -> Result<bool, String>
where
    F: FnMut(Vec<u8>) -> Fut,
    Fut: Future<Output = bool>,
{
    let mut given: u64 = 0;
    let mut etag: Option<String> = None;
    let mut failures = 0;
    loop {
        match attempt(client, request, &mut given, &mut etag, &mut give).await {
            Ok(true) => return Ok(true),
            Ok(false) => return Ok(false),
            Err(Attempt::Cut(_)) if failures < retries => {
                wait(failures).await;
                failures += 1;
            }
            Err(Attempt::Cut(reason)) | Err(Attempt::Failed(reason)) => return Err(reason),
        }
    }
}

/// Makes one attempt of a download, from `given` bytes, pinned to `etag` once known.
async fn attempt<F, Fut>(
    client: &Client,
    request: &PreparedRequest,
    given: &mut u64,
    etag: &mut Option<String>,
    give: &mut F,
) -> Result<bool, Attempt>
where
    F: FnMut(Vec<u8>) -> Fut,
    Fut: Future<Output = bool>,
{
    let mut extra = Vec::new();
    if *given > 0 {
        match etag {
            Some(etag) => {
                extra.push(("Range", format!("bytes={given}-")));
                extra.push(("If-Match", etag.clone()));
            }
            None => {
                return Err(Attempt::Failed(
                    "the download was cut, and storage gave no entity tag to resume it".to_string(),
                ))
            }
        }
    }
    let (conn, url) = connection(client, request, &extra).map_err(Attempt::Failed)?;
    let mut conn = conn
        .await
        .map_err(|err| Attempt::Cut(redact(format!("connection failed: {err}"), &url)))?;
    let status = conn.status().map(|status| status as u16).unwrap_or(0);

    let expected = if *given == 0 {
        if !(200..300).contains(&status) {
            return Err(answer_error(&mut conn, status).await);
        }
        *etag = header(&conn, "etag");
        header(&conn, "content-length").and_then(|length| length.parse::<u64>().ok())
    } else {
        match status {
            206 => {
                let range = header(&conn, "content-range").unwrap_or_default();
                if !range.starts_with(&format!("bytes {given}-")) {
                    return Err(Attempt::Failed(
                        "storage resumed the download from another position".to_string(),
                    ));
                }
                header(&conn, "content-length")
                    .and_then(|length| length.parse::<u64>().ok())
                    .map(|length| *given + length)
            }
            412 => {
                return Err(Attempt::Failed(
                    "the object changed while it was downloaded".to_string(),
                ))
            }
            200 => {
                return Err(Attempt::Failed(
                    "storage did not resume the download".to_string(),
                ))
            }
            status => return Err(answer_error(&mut conn, status).await),
        }
    };

    let mut body = conn.response_body();
    loop {
        let mut chunk = vec![0; CHUNK];
        match body.read(&mut chunk).await {
            Ok(0) => break,
            Ok(read) => {
                chunk.truncate(read);
                *given += read as u64;
                if !give(chunk).await {
                    return Ok(false);
                }
            }
            Err(err) => {
                return Err(Attempt::Cut(redact(
                    format!("the download was cut: {err}"),
                    &url,
                )))
            }
        }
    }
    match expected {
        Some(expected) if *given < expected => Err(Attempt::Cut(format!(
            "the download was cut after {given} of {expected} bytes"
        ))),
        _ => Ok(true),
    }
}

/// Error for an answer of `status`, worth another attempt or not.
async fn answer_error(conn: &mut Conn, status: u16) -> Attempt {
    let reason = refusal(status, &error_body(conn).await);
    if retriable(status) {
        Attempt::Cut(reason)
    } else {
        Attempt::Failed(reason)
    }
}

#[cfg(all(test, feature = "real"))]
pub(crate) mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};

    /// A request received by the test server: its first line and headers.
    #[derive(Debug, Clone)]
    pub(crate) struct Received {
        pub line: String,
        pub headers: BTreeMap<String, String>,
        pub body: Vec<u8>,
    }

    /// What the test server does with a request: the bytes it writes back, then whether it
    /// closes the connection right after.
    pub(crate) type Respond = Box<dyn FnMut(usize, &Received) -> Vec<u8> + Send>;

    /// Serves requests on a local port with `respond`, keeping what it received.
    pub(crate) fn serve(mut respond: Respond) -> (u16, Arc<Mutex<Vec<Received>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let received = Arc::new(Mutex::new(Vec::new()));
        let kept = Arc::clone(&received);
        std::thread::spawn(move || {
            for (index, stream) in listener.incoming().enumerate() {
                let Ok(mut stream) = stream else { break };
                let request = read_request(&mut stream);
                kept.lock().unwrap().push(request.clone());
                let _ = stream.write_all(&respond(index, &request));
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        });
        (port, received)
    }

    fn read_request(stream: &mut TcpStream) -> Received {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let mut headers = BTreeMap::new();
        loop {
            let mut header = String::new();
            reader.read_line(&mut header).unwrap();
            let header = header.trim_end();
            if header.is_empty() {
                break;
            }
            if let Some((name, value)) = header.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
            }
        }
        let length = headers
            .get("content-length")
            .and_then(|length| length.parse::<usize>().ok())
            .unwrap_or(0);
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        Received {
            line: line.trim_end().to_string(),
            headers,
            body,
        }
    }

    /// An HTTP answer of `status` with `headers` and `body`, its length being `length`.
    pub(crate) fn answer(
        status: &str,
        headers: &[(&str, String)],
        body: &[u8],
        length: usize,
    ) -> Vec<u8> {
        let mut answer =
            format!("HTTP/1.1 {status}\r\nContent-Length: {length}\r\nConnection: close\r\n");
        for (name, value) in headers {
            answer.push_str(&format!("{name}: {value}\r\n"));
        }
        answer.push_str("\r\n");
        let mut answer = answer.into_bytes();
        answer.extend_from_slice(body);
        answer
    }

    pub(crate) fn request(method: &str, port: u16) -> PreparedRequest {
        PreparedRequest {
            method: method.to_string(),
            url: format!("http://127.0.0.1:{port}/object?X-Amz-Signature=sentinel-signature"),
            headers: BTreeMap::from([("x-prepared".to_string(), "yes".to_string())]),
        }
    }

    fn client() -> Client {
        Client::new(trillium_smol::ClientConfig::new())
    }

    fn run(request: &PreparedRequest, retries: u32) -> (Result<bool, String>, Vec<u8>) {
        smol::block_on(async {
            let mut got = Vec::new();
            let result = download(&client(), request, retries, |bytes| {
                got.extend(bytes);
                async { true }
            })
            .await;
            (result, got)
        })
    }

    const OBJECT: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    #[test]
    fn downloads_resume_after_a_cut() {
        let (port, received) = serve(Box::new(|index, request| match index {
            // Cut after 10 bytes of 36.
            0 => answer(
                "200 OK",
                &[("ETag", "\"v1\"".to_string())],
                &OBJECT[..10],
                OBJECT.len(),
            ),
            _ => {
                let start: usize = request.headers["range"]
                    .trim_start_matches("bytes=")
                    .trim_end_matches('-')
                    .parse()
                    .unwrap();
                answer(
                    "206 Partial Content",
                    &[
                        ("ETag", "\"v1\"".to_string()),
                        ("Content-Range", format!("bytes {start}-35/36")),
                    ],
                    &OBJECT[start..],
                    OBJECT.len() - start,
                )
            }
        }));
        let (result, got) = run(&request("GET", port), 2);
        assert_eq!(result, Ok(true));
        assert_eq!(got, OBJECT);

        let received = received.lock().unwrap();
        assert_eq!(received.len(), 2);
        assert_eq!(
            received[0].line,
            "GET /object?X-Amz-Signature=sentinel-signature HTTP/1.1"
        );
        assert!(received[0].body.is_empty());
        assert_eq!(received[0].headers["x-prepared"], "yes");
        assert!(!received[0].headers.contains_key("range"));
        assert_eq!(received[1].headers["range"], "bytes=10-");
        assert_eq!(received[1].headers["if-match"], "\"v1\"");
    }

    #[test]
    fn changed_objects_fail() {
        let (port, _) = serve(Box::new(|index, _| match index {
            0 => answer(
                "200 OK",
                &[("ETag", "\"v1\"".to_string())],
                &OBJECT[..10],
                OBJECT.len(),
            ),
            _ => answer("412 Precondition Failed", &[], b"", 0),
        }));
        let (result, got) = run(&request("GET", port), 2);
        assert_eq!(
            result,
            Err("the object changed while it was downloaded".to_string())
        );
        assert_eq!(got, &OBJECT[..10]);
    }

    #[test]
    fn wrong_ranges_fail() {
        let (port, _) = serve(Box::new(|index, _| match index {
            0 => answer(
                "200 OK",
                &[("ETag", "\"v1\"".to_string())],
                &OBJECT[..10],
                OBJECT.len(),
            ),
            _ => answer(
                "206 Partial Content",
                &[("Content-Range", "bytes 0-35/36".to_string())],
                OBJECT,
                OBJECT.len(),
            ),
        }));
        let (result, _) = run(&request("GET", port), 2);
        assert_eq!(
            result,
            Err("storage resumed the download from another position".to_string())
        );
    }

    #[test]
    fn refusals_give_status_and_code_only() {
        let body = b"<?xml version=\"1.0\"?><Error><Code>AccessDenied</Code><Message>Request has expired for http://127.0.0.1/object?X-Amz-Signature=sentinel-signature</Message></Error>";
        let (port, received) = serve(Box::new(move |_, _| {
            answer("403 Forbidden", &[], body, body.len())
        }));
        let (result, _) = run(&request("GET", port), 2);
        assert_eq!(
            result,
            Err("storage answered 403 (AccessDenied)".to_string())
        );
        // Refusals are not worth another attempt.
        assert_eq!(received.lock().unwrap().len(), 1);
    }

    #[test]
    fn retries_end() {
        let (port, received) = serve(Box::new(|_, _| {
            answer("503 Service Unavailable", &[], b"", 0)
        }));
        let (result, _) = run(&request("GET", port), 1);
        assert_eq!(result, Err("storage answered 503".to_string()));
        assert_eq!(received.lock().unwrap().len(), 2);
    }

    #[test]
    fn urls_are_redacted() {
        let url = Url::parse("https://storage.example/object?X-Amz-Signature=sentinel").unwrap();
        let message = redact(
            "failed for https://storage.example/object?X-Amz-Signature=sentinel and X-Amz-Signature=sentinel".to_string(),
            &url,
        );
        assert!(!message.contains("sentinel"), "{message}");
        assert_eq!(
            refusal(500, b"<Code>bad code with spaces</Code>"),
            "storage answered 500"
        );
    }
}
