//! The `http` package downloads what prepared requests give, resuming after cuts.

use super::common;
use melodium_common::executive::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

const DOWNLOAD: &str = include_str!("scripts/http_download.mel");
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
