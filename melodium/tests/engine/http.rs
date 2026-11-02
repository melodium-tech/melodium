//! The `http` package downloads what prepared requests give, resuming after cuts, and
//! uploads data to them, in parts or in one request.

use super::common;
use melodium_common::executive::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

const DOWNLOAD: &str = include_str!("scripts/http_download.mel");
const UPLOAD: &str = include_str!("scripts/http_upload.mel");
const OBJECT: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// Serves `OBJECT`, cutting the first answer after 10 bytes, and resuming from the range
/// asked next. Gives the headers of each request received.
fn serve() -> (u16, std::sync::mpsc::Receiver<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for (index, stream) in listener.incoming().enumerate() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut headers = Vec::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                headers.push(line.trim().to_ascii_lowercase());
            }
            let start = headers
                .iter()
                .find_map(|header| header.strip_prefix("range: bytes="))
                .map(|range| range.trim_end_matches('-').parse::<usize>().unwrap());
            let _ = sender.send(headers);
            let answer = match (index, start) {
                (0, _) => [
                    format!("HTTP/1.1 200 OK\r\nContent-Length: 36\r\nETag: \"v1\"\r\nConnection: close\r\n\r\n").into_bytes(),
                    OBJECT[..10].to_vec(),
                ]
                .concat(),
                (_, Some(start)) => [
                    format!(
                        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-35/36\r\nETag: \"v1\"\r\nConnection: close\r\n\r\n",
                        36 - start
                    )
                    .into_bytes(),
                    OBJECT[start..].to_vec(),
                ]
                .concat(),
                _ => b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n".to_vec(),
            };
            let _ = stream.write_all(&answer);
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    });
    (port, receiver)
}

#[test]
fn prepared_downloads_resume() {
    let (port, received) = serve();
    let url = format!("http://127.0.0.1:{port}/object?X-Amz-Signature=sentinel-signature");
    let (logs, events) = common::run(
        DOWNLOAD,
        HashMap::from([("url".to_string(), Value::String(url))]),
    );

    let data: Vec<u8> = common::sent(&events, "download", "data")
        .into_iter()
        .map(|value| match value {
            Value::Byte(byte) => byte,
            other => panic!("not a byte: {other:?}"),
        })
        .collect();
    assert_eq!(data, OBJECT);
    assert_eq!(common::sent(&events, "download", "completed").len(), 1);
    assert!(common::log(&logs, "download").is_none(), "{logs:?}");

    let first = received.recv().unwrap();
    assert!(first.contains(&"x-prepared: yes".to_string()), "{first:?}");
    let second = received.recv().unwrap();
    assert!(
        second.contains(&"range: bytes=10-".to_string()),
        "{second:?}"
    );
    assert!(
        second.contains(&"if-match: \"v1\"".to_string()),
        "{second:?}"
    );

    assert_eq!(common::message(&logs, "shown"), "GET 127.0.0.1");
    for log in &logs {
        assert!(!log.message.contains("sentinel"), "{}", log.message);
    }
}

/// A request received by `serve_uploads`: its headers and body.
struct Upload {
    headers: Vec<String>,
    body: Vec<u8>,
}

/// Accepts uploads, refusing the first part with `503` (a single upload is never sent
/// again), answering an entity tag naming the `X-Part` header of each request.
fn serve_uploads() -> (u16, std::sync::mpsc::Receiver<Upload>) {
    use std::io::Read;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut refused = false;
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut headers = Vec::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                headers.push(line.trim().to_ascii_lowercase());
            }
            let header = |name: &str| {
                headers
                    .iter()
                    .find_map(|header| header.strip_prefix(&format!("{name}: ")))
                    .map(str::to_string)
            };
            if header("expect").as_deref() == Some("100-continue") {
                let _ = stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
            }
            let length = header("content-length")
                .map(|length| length.parse::<usize>().unwrap())
                .unwrap_or(0);
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let part = header("x-part").unwrap_or_default();
            let _ = sender.send(Upload { headers, body });
            let answer = if part == "1" && !refused {
                refused = true;
                "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_string()
            } else {
                format!("HTTP/1.1 200 OK\r\nETag: \"tag-{part}\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            };
            let _ = stream.write_all(answer.as_bytes());
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    });
    (port, receiver)
}

#[test]
fn prepared_uploads_are_sent() {
    let (port, received) = serve_uploads();
    let url = format!("http://127.0.0.1:{port}/object?X-Amz-Signature=sentinel-signature");
    let (logs, events) = common::run(
        UPLOAD,
        HashMap::from([("url".to_string(), Value::String(url))]),
    );

    assert!(common::log(&logs, "parts").is_none(), "{logs:?}");
    assert!(common::log(&logs, "put").is_none(), "{logs:?}");
    assert_eq!(common::sent(&events, "parts", "completed").len(), 1);
    assert_eq!(common::sent(&events, "put", "completed").len(), 1);
    assert_eq!(common::message(&logs, "tag"), "\"tag-1\"");
    assert_eq!(
        common::message(&logs, "part size"),
        (16u64 << 20).to_string()
    );

    // The refused one is sent again, so that three requests are received.
    let uploads: Vec<Upload> = (0..3).map(|_| received.recv().unwrap()).collect();
    let part = |name: &str| {
        uploads
            .iter()
            .filter(|upload| upload.headers.contains(&format!("x-part: {name}")))
            .collect::<Vec<_>>()
    };
    let part_uploads = part("1");
    let single_uploads = part("single");
    assert_eq!(part_uploads.len() + single_uploads.len(), 3);
    for upload in &part_uploads {
        assert_eq!(upload.body, b"uploaded in parts");
    }
    for upload in &single_uploads {
        assert_eq!(upload.body, b"single");
        assert!(upload.headers.contains(&"content-length: 6".to_string()));
    }
}
