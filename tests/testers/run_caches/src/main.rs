//! Programs run by api.melodium.tech restore directories as named caches and save them as new
//! versions, assembled in Mélodium (`work/cache`): archives are streamed through upload
//! sessions in parts presigned one by one, retried after refusals, and downloads are resumed
//! after cuts, then checked against the sha256 recorded. A miss is never an error, a tampered
//! archive never reaches its target, a cache holding a revealed secret value is aborted, and no
//! presigned URL or token appears in what runs show.
//!
//! The API and storage are a mock following the contract of api.melodium.tech (upload sessions,
//! caches as names with versions), refusing a part once and cutting a download once.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use tester::{melodium, temp_file, temp_path};

const SAVE_SCRIPT: &str = include_str!("../../../run_caches_save.mel");
const RESTORE_SCRIPT: &str = include_str!("../../../run_caches_restore.mel");
const LEAK_SCRIPT: &str = include_str!("../../../run_caches_leak.mel");
const STEP_SCRIPT: &str = include_str!("../../../run_caches_step.mel");

const RUN_ID: &str = "15415415-0154-0154-0154-154154154154";
const RUN_TOKEN: &str = "cad_v_run-caches-run-token";
const SIGNATURE: &str = "presigned-signature-sentinel";
const PASSWORD: &str = "run-caches-password-sentinel";
const MIB: u64 = 1024 * 1024;

/// Current version of a cache.
#[derive(Clone)]
struct Cache {
    version: String,
    sha256: String,
    object: String,
}

/// Upload session.
#[derive(Default)]
struct Upload {
    name: String,
    presigned: Vec<u64>,
    parts: HashMap<u64, Vec<u8>>,
    tags: HashMap<u64, String>,
}

#[derive(Default)]
struct State {
    /// Requests received, as method and path without the query, with their authorization.
    requests: Vec<(String, Option<String>)>,
    caches: HashMap<String, Cache>,
    objects: HashMap<String, Vec<u8>>,
    uploads: HashMap<String, Upload>,
    aborted: Vec<String>,
    /// Whether the storage of the organization is full.
    full: bool,
    /// Parts whose first upload was refused, to be retried.
    refused_parts: Vec<String>,
    /// Objects whose first download was cut.
    cut_downloads: Vec<String>,
}

/// Mock of the API and of storage.
struct Mock {
    address: String,
    state: Arc<Mutex<State>>,
}

/// Request received by the mock.
struct Request {
    method: String,
    path: String,
    query: String,
    authorization: Option<String>,
    range: Option<usize>,
    body: Vec<u8>,
}

/// Reads a request, answering `100 Continue` when asked to, with a body of known length or
/// chunked.
fn read_request(stream: &mut TcpStream, reader: &mut impl BufRead) -> Request {
    let mut request_line = String::new();
    let _ = reader.read_line(&mut request_line);
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let target = parts.next().unwrap_or_default().to_string();
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_string(), query.to_string()),
        None => (target, String::new()),
    };
    let mut authorization = None;
    let mut range = None;
    let mut length = 0;
    let mut chunked = false;
    let mut expect = false;
    loop {
        let mut line = String::new();
        let _ = reader.read_line(&mut line);
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();
            if name.eq_ignore_ascii_case("authorization") {
                authorization = Some(value.to_string());
            } else if name.eq_ignore_ascii_case("content-length") {
                length = value.parse().unwrap_or(0);
            } else if name.eq_ignore_ascii_case("transfer-encoding") {
                chunked = value.eq_ignore_ascii_case("chunked");
            } else if name.eq_ignore_ascii_case("expect") {
                expect = value.eq_ignore_ascii_case("100-continue");
            } else if name.eq_ignore_ascii_case("range") {
                range = value
                    .strip_prefix("bytes=")
                    .and_then(|range| range.strip_suffix('-'))
                    .and_then(|start| start.parse().ok());
            }
        }
    }
    if expect {
        let _ = stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
    }
    let mut body = Vec::new();
    if chunked {
        loop {
            let mut size = String::new();
            let _ = reader.read_line(&mut size);
            let size = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
            if size == 0 {
                let mut end = String::new();
                let _ = reader.read_line(&mut end);
                break;
            }
            let mut chunk = vec![0; size];
            let _ = reader.read_exact(&mut chunk);
            body.extend(chunk);
            let mut end = String::new();
            let _ = reader.read_line(&mut end);
        }
    } else {
        body = vec![0; length];
        let _ = reader.read_exact(&mut body);
    }
    Request {
        method,
        path,
        query,
        authorization,
        range,
        body,
    }
}

fn presigned(address: &str, path: &str, method: &str) -> Value {
    json!({
        "method": method,
        "url": format!("{address}{path}?X-Amz-Expires=900&X-Amz-Signature={SIGNATURE}"),
        "headers": {}
    })
}

/// Answers a request to the API, or `None` for storage.
fn api(state: &mut State, base: &str, request: &Request, next: &mut usize) -> Option<(u16, Value)> {
    let path = request
        .path
        .strip_prefix(&format!("/execution/run/{RUN_ID}/"))?;
    if request.authorization.as_deref() != Some(&format!("Bearer {RUN_TOKEN}")) {
        return Some((403, json!({"error": "not this run's machine token"})));
    }
    let body: Value = serde_json::from_slice(&request.body).unwrap_or(Value::Null);
    let segments: Vec<&str> = path.split('/').collect();
    Some(match (request.method.as_str(), segments.as_slice()) {
        ("POST", ["cache", "restore"]) => {
            let name = body["name"].as_str().unwrap_or_default();
            match state.caches.get(name) {
                Some(cache) => (
                    200,
                    json!({"hit": {
                        "name": name,
                        "version": cache.version,
                        "size": state.objects[&cache.object].len(),
                        "sha256": cache.sha256,
                        "download": presigned(base, &format!("/storage/{}", cache.object), "GET"),
                    }}),
                ),
                None => (200, json!("miss")),
            }
        }
        ("POST", ["storage", "uploads"]) => {
            let name = body["purpose"]["cache"]["name"].as_str();
            match name {
                _ if state.full => (
                    507,
                    json!({"error": "The organization's storage is full even without its caches"}),
                ),
                Some(name) => {
                    *next += 1;
                    let upload_id = format!("00000000-0000-0000-0000-{:012}", *next);
                    state.uploads.insert(
                        upload_id.clone(),
                        Upload {
                            name: name.to_string(),
                            ..Default::default()
                        },
                    );
                    (200, json!({"upload_id": upload_id, "upload": "parts"}))
                }
                None => (400, json!({"error": "invalid purpose"})),
            }
        }
        ("POST", ["storage", "uploads", upload_id, "parts"]) => {
            let Some(upload) = state.uploads.get_mut(*upload_id) else {
                return Some((403, json!({"error": "not an upload this run opened"})));
            };
            let first = body["first"].as_u64().unwrap_or(0);
            let count = body["count"].as_u64().unwrap_or(0);
            let part_size = body["part_size"].as_u64().unwrap_or(0);
            if !(1..=64).contains(&count)
                || first == 0
                || !(5 * MIB..=5 * 1024 * MIB).contains(&part_size)
            {
                return Some((400, json!({"error": "invalid batch"})));
            }
            let parts: Vec<Value> = (first..first + count)
                .map(|number| {
                    upload.presigned.push(number);
                    json!({
                        "part_number": number,
                        "request": presigned(base, &format!("/storage/{upload_id}/part/{number}"), "PUT"),
                    })
                })
                .collect();
            (200, json!({"tag_header": "etag", "parts": parts}))
        }
        ("POST", ["storage", "uploads", upload_id, "complete"]) => {
            let Some(upload) = state.uploads.remove(*upload_id) else {
                return Some((403, json!({"error": "not an upload this run opened"})));
            };
            let mut parts: Vec<(u64, String)> = body["parts"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|part| {
                    (
                        part["part_number"].as_u64().unwrap_or(0),
                        part["etag"].as_str().unwrap_or_default().to_string(),
                    )
                })
                .collect();
            parts.sort();
            let whole = parts.iter().enumerate().all(|(index, (number, etag))| {
                *number == index as u64 + 1 && upload.tags.get(number) == Some(etag)
            });
            let content: Vec<u8> = parts
                .iter()
                .flat_map(|(number, _)| upload.parts.get(number).cloned().unwrap_or_default())
                .collect();
            if !whole || parts.is_empty() || body["size"].as_u64() != Some(content.len() as u64) {
                return Some((400, json!({"error": "missing parts, or another size"})));
            }
            let sha256 = body["sha256"].as_str().unwrap_or_default().to_string();
            match state.caches.get(&upload.name) {
                Some(cache) if cache.sha256 == sha256 => (200, json!("unchanged")),
                _ => {
                    state.objects.insert(upload_id.to_string(), content);
                    state.caches.insert(
                        upload.name.clone(),
                        Cache {
                            version: upload_id.to_string(),
                            sha256,
                            object: upload_id.to_string(),
                        },
                    );
                    (
                        200,
                        json!({"cache": {"name": upload.name, "version": upload_id}}),
                    )
                }
            }
        }
        ("POST", ["storage", "uploads", upload_id, "abort"]) => {
            state.uploads.remove(*upload_id);
            state.aborted.push(upload_id.to_string());
            (200, json!({"success": true}))
        }
        _ => (404, json!({"error": "not found"})),
    })
}

/// Starts a mock of the API giving caches, and of storage.
fn api_server() -> Mock {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let state = Arc::new(Mutex::new(State::default()));

    let shared = Arc::clone(&state);
    let base = address.clone();
    std::thread::spawn(move || {
        let next = Arc::new(Mutex::new(0));
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let shared = Arc::clone(&shared);
            let base = base.clone();
            let next = Arc::clone(&next);
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let request = read_request(&mut stream, &mut reader);
                let mut state = shared.lock().unwrap();
                state.requests.push((
                    format!("{} {}", request.method, request.path),
                    request.authorization.clone(),
                ));

                if let Some((status, body)) =
                    api(&mut state, &base, &request, &mut next.lock().unwrap())
                {
                    drop(state);
                    let body = body.to_string();
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        if status == 200 { "OK" } else { "Refused" },
                        body.len()
                    );
                    return;
                }

                let target = request.path.strip_prefix("/storage/").unwrap_or_default();
                let mut cut = false;
                let mut etag = None;
                let (status, body): (&str, Vec<u8>) = if !request
                    .query
                    .contains(&format!("X-Amz-Signature={SIGNATURE}"))
                {
                    (
                        "403 Forbidden",
                        b"<Error><Code>AccessDenied</Code></Error>".to_vec(),
                    )
                } else if request.method == "PUT" {
                    match target.split_once("/part/") {
                        // The first upload of the second part is refused, then accepted.
                        Some((_, "2")) if !state.refused_parts.contains(&target.to_string()) => {
                            state.refused_parts.push(target.to_string());
                            (
                                "503 Service Unavailable",
                                b"<Error><Code>SlowDown</Code></Error>".to_vec(),
                            )
                        }
                        Some((upload_id, number)) => {
                            let number: u64 = number.parse().unwrap_or(0);
                            match state.uploads.get_mut(upload_id) {
                                Some(upload) if upload.presigned.contains(&number) => {
                                    let tag = format!("\"part-{number}-{}\"", request.body.len());
                                    upload.parts.insert(number, request.body.clone());
                                    upload.tags.insert(number, tag.clone());
                                    etag = Some(tag);
                                    ("200 OK", Vec::new())
                                }
                                _ => (
                                    "403 Forbidden",
                                    b"<Error><Code>AccessDenied</Code></Error>".to_vec(),
                                ),
                            }
                        }
                        None => ("404 Not Found", Vec::new()),
                    }
                } else {
                    let content = state.objects.get(target).cloned().unwrap_or_default();
                    etag = Some(format!("\"object-{}\"", content.len()));
                    let start = request.range.unwrap_or(0);
                    // The first download of each object is cut halfway.
                    if !state.cut_downloads.contains(&target.to_string())
                        && start == 0
                        && content.len() > 1
                    {
                        state.cut_downloads.push(target.to_string());
                        cut = true;
                    }
                    if start > 0 {
                        let _ = write!(
                            stream,
                            "HTTP/1.1 206 Partial Content\r\nETag: {}\r\nContent-Range: bytes {start}-{}/{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            etag.unwrap(),
                            content.len() - 1,
                            content.len(),
                            content.len() - start
                        );
                        let _ = stream.write_all(&content[start..]);
                        return;
                    }
                    ("200 OK", content)
                };
                drop(state);

                let etag = etag
                    .map(|etag| format!("ETag: {etag}\r\n"))
                    .unwrap_or_default();
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status}\r\n{etag}Content-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                if cut {
                    let _ = stream.write_all(&body[..body.len() / 2]);
                    let _ = stream.flush();
                } else {
                    let _ = stream.write_all(&body);
                }
            });
        }
    });

    Mock { address, state }
}

/// Fresh volume directory, holding `target` with the files given.
fn volume(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let path = temp_path("run_caches", name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(path.join("target")).unwrap();
    for (file, content) in files {
        let file = path.join("target").join(file);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    path
}

/// Files of a dependency cache, needing three parts once archived and compressed.
fn dependencies() -> Vec<(&'static str, Vec<u8>)> {
    // Barely compressible content.
    let mut state: u64 = 0x2545F4914F6CDD1D;
    let mut noise = Vec::with_capacity(20 * MIB as usize);
    for _ in 0..20 * MIB {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        noise.push(state as u8);
    }
    vec![
        ("deps/libserde.rlib", noise),
        ("deps/nested/libuuid.rlib", b"compiled uuid".to_vec()),
        (
            "CACHEDIR.TAG",
            b"Signature: 8a477f597d28d172789f06886806bc55".to_vec(),
        ),
    ]
}

/// Runs `script` with the volume `data` at `volume`, configured as on an api.melodium.tech worker
/// when an API address is given, and as elsewhere otherwise.
fn run(script: &Path, api: Option<&str>, volume: &Path, args: &[&str]) -> Output {
    let mut command: Command = melodium();
    command
        .arg("run")
        .arg(script)
        .args(args)
        .current_dir(volume)
        .env_remove("MELODIUM_API_URL")
        .env_remove("MELODIUM_API_TOKEN")
        .env_remove("MELODIUM_RUN_ID")
        .env("MELODIUM_RUN_VOLUMES", "data")
        .env("MELODIUM_RUN_VOLUME_data", volume)
        .env("MELODIUM_SECRET_CACHE_PASSWORD", PASSWORD);
    if let Some(address) = api {
        command
            .env("MELODIUM_API_URL", address)
            .env("MELODIUM_API_TOKEN", RUN_TOKEN)
            .env("MELODIUM_RUN_ID", RUN_ID);
    }
    command.output().unwrap()
}

fn assert_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "expected {:?} in:\n{}",
        expected,
        text
    );
}

/// Checks that what a run shows holds no credential.
fn assert_no_credentials(output: &Output) {
    for text in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(text);
        for credential in [SIGNATURE, RUN_TOKEN, PASSWORD] {
            assert!(!text.contains(credential), "{credential} shown in:\n{text}");
        }
    }
}

fn assert_same_files(expected: &Path, actual: &Path) {
    for (file, content) in dependencies() {
        assert_eq!(
            std::fs::read(actual.join(file)).unwrap(),
            content,
            "{file} restored from {}",
            expected.display()
        );
    }
}

/// Targets of the requests received since `from`.
fn requests(mock: &Mock, from: usize) -> Vec<String> {
    mock.state.lock().unwrap().requests[from..]
        .iter()
        .map(|(target, _)| target.replace(RUN_ID, "RUN"))
        .collect()
}

fn caches_are_saved_then_restored() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let mock = api_server();
    let saved = volume("saved", &dependencies());

    let output = run(&save, Some(&mock.address), &saved, &["--name", "cargo"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: saved");
    assert_no_credentials(&output);
    {
        let state = mock.state.lock().unwrap();
        assert!(state.caches.contains_key("cargo"));
        assert_eq!(
            state.refused_parts.len(),
            1,
            "a part was refused once, then sent again"
        );
        let parts = state
            .requests
            .iter()
            .filter(|(target, _)| target.ends_with("/parts"))
            .count();
        assert_eq!(parts, 3, "each part presigned alone");
        assert!(state
            .requests
            .iter()
            .filter(|(target, _)| target.starts_with("POST /execution/run/"))
            .all(|(_, authorization)| authorization.as_deref()
                == Some(&format!("Bearer {RUN_TOKEN}"))));
        // Storage never gets the token of the run.
        assert!(state
            .requests
            .iter()
            .filter(|(target, _)| target.contains(" /storage/"))
            .all(|(_, authorization)| authorization.is_none()));
    }

    // A later run restores it under its name, replacing what was there.
    let restored = volume("restored", &[("stale.txt", b"stale".to_vec())]);
    let output = run(
        &restore,
        Some(&mock.address),
        &restored,
        &["--name", "cargo", "--fallback", "none"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: restored");
    assert_no_credentials(&output);
    assert_same_files(&saved, &restored.join("target"));
    assert!(!restored.join("target/stale.txt").exists());
    assert!(!restored.join("target.melodium-restore").exists());
    assert_eq!(
        mock.state.lock().unwrap().cut_downloads.len(),
        1,
        "the download was cut once, then resumed"
    );

    // A name missing falls back to another.
    let fallback = volume("fallback", &[]);
    let output = run(
        &restore,
        Some(&mock.address),
        &fallback,
        &["--name", "cargo-new", "--fallback", "cargo"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(
        &stdout,
        "Cache `cargo-new` not restored: api.melodium.tech has none of that name",
    );
    assert_contains(&stdout, "cache: restored fallback");
    assert_same_files(&saved, &fallback.join("target"));

    for volume in [saved, restored, fallback] {
        let _ = std::fs::remove_dir_all(&volume);
    }
}

/// The same content saved again gives the same archive: api.melodium.tech keeps the current
/// version.
fn identical_saves_keep_the_current_version() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let mock = api_server();
    let first = volume("identical", &dependencies());

    let output = run(&save, Some(&mock.address), &first, &["--name", "cargo"]);
    assert!(output.status.success(), "{:?}", output);
    let version = mock.state.lock().unwrap().caches["cargo"].version.clone();

    let output = run(&save, Some(&mock.address), &first, &["--name", "cargo"]);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&String::from_utf8_lossy(&output.stdout), "cache: saved");
    assert_eq!(mock.state.lock().unwrap().caches["cargo"].version, version);

    // Other content makes a new version.
    std::fs::write(first.join("target/deps/new.rlib"), b"new").unwrap();
    let output = run(&save, Some(&mock.address), &first, &["--name", "cargo"]);
    assert!(output.status.success(), "{:?}", output);
    assert_ne!(mock.state.lock().unwrap().caches["cargo"].version, version);

    let _ = std::fs::remove_dir_all(&first);
}

fn misses_are_not_errors() {
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let mock = api_server();
    let empty = volume("miss", &[("kept.txt", b"kept".to_vec())]);

    let output = run(
        &restore,
        Some(&mock.address),
        &empty,
        &["--name", "cargo", "--fallback", "cargo-old"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: missed");
    assert!(!stdout.contains("error"), "{}", stdout);
    assert_eq!(
        std::fs::read(empty.join("target/kept.txt")).unwrap(),
        b"kept"
    );

    let _ = std::fs::remove_dir_all(&empty);
}

/// A full storage refuses the session: the save fails with the reason of the API, nothing
/// being uploaded.
fn full_storage_refuses_saves() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let mock = api_server();
    mock.state.lock().unwrap().full = true;
    let full = volume("full", &dependencies());

    let output = run(&save, Some(&mock.address), &full, &["--name", "cargo"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "cache: not saved");
    assert_contains(
        &stdout,
        "api.melodium.tech answered 507 Insufficient Storage: {\"error\":\"The organization's storage is full even without its caches\"}",
    );
    assert_eq!(
        requests(&mock, 0),
        ["POST /execution/run/RUN/storage/uploads"]
    );

    let _ = std::fs::remove_dir_all(&full);
}

fn tampered_archives_are_never_extracted() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let mock = api_server();
    let saved = volume("tampered_saved", &dependencies());

    let output = run(&save, Some(&mock.address), &saved, &["--name", "cargo"]);
    assert!(output.status.success(), "{:?}", output);
    // The archive is altered in storage, its sha256 staying the one recorded.
    {
        let mut state = mock.state.lock().unwrap();
        let object = state.caches["cargo"].object.clone();
        let content = state.objects.get_mut(&object).unwrap();
        let middle = content.len() / 2;
        content[middle] ^= 0xff;
    }

    let restored = volume("tampered_restored", &[("kept.txt", b"kept".to_vec())]);
    let output = run(
        &restore,
        Some(&mock.address),
        &restored,
        &["--name", "cargo", "--fallback", "none"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: missed");
    assert_contains(&stdout, "Cache `cargo` not restored:");
    assert_eq!(
        std::fs::read(restored.join("target/kept.txt")).unwrap(),
        b"kept"
    );
    assert!(!restored.join("target/deps").exists());
    assert!(!restored.join("target.melodium-restore").exists());
    assert_no_credentials(&output);

    let _ = std::fs::remove_dir_all(&saved);
    let _ = std::fs::remove_dir_all(&restored);
}

#[cfg(unix)]
fn revealed_values_never_leave_in_caches() {
    let leak = temp_file("run_caches", "leak.mel", LEAK_SCRIPT);
    let mock = api_server();
    let leaking = volume("leak", &dependencies());

    let output = run(
        &leak,
        Some(&mock.address),
        &leaking,
        &["--name", "config", "--directory", leaking.to_str().unwrap()],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        std::fs::read_to_string(leaking.join("target/config"))
            .unwrap()
            .contains(PASSWORD),
        "the command got the password"
    );
    assert_contains(
        &stdout,
        "the data holds a value revealed from secret 'env:MELODIUM_SECRET_CACHE_PASSWORD'",
    );
    assert!(!stdout.contains("cache: saved"), "{}", stdout);
    let state = mock.state.lock().unwrap();
    assert!(state.caches.is_empty());
    assert_eq!(state.aborted.len(), 1, "the session is aborted");
    drop(state);
    assert_no_credentials(&output);

    let _ = std::fs::remove_dir_all(&leaking);
}

/// The token of the run is masked from the start: a cache holding it is aborted too.
fn run_tokens_never_leave_in_caches() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let mock = api_server();
    let leaking = volume(
        "token",
        &[(
            "env",
            format!("MELODIUM_API_TOKEN={RUN_TOKEN}\n").into_bytes(),
        )],
    );

    let output = run(&save, Some(&mock.address), &leaking, &["--name", "env"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(
        &stdout,
        "the data holds a value revealed from secret 'api-run:token'",
    );
    assert!(mock.state.lock().unwrap().caches.is_empty());
    assert_no_credentials(&output);

    let _ = std::fs::remove_dir_all(&leaking);
}

#[cfg(unix)]
fn steps_restore_before_and_save_after_commands() {
    let step = temp_file("run_caches", "step.mel", STEP_SCRIPT);
    let mock = api_server();
    let step_run = |volume: &Path, local: bool, exit: &str| {
        run(
            &step,
            Some(&mock.address),
            volume,
            &[
                "--directory",
                volume.to_str().unwrap(),
                "--local",
                if local { "true" } else { "false" },
                "--exit",
                exit,
            ],
        )
    };
    let completed = |mock: &Mock| {
        mock.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|(target, _)| target.ends_with("/complete"))
            .count()
    };
    let fresh = |name: &str, lock: &str| {
        let volume = volume(name, &[]);
        std::fs::remove_dir_all(volume.join("target")).unwrap();
        std::fs::write(volume.join("lock"), lock).unwrap();
        volume
    };

    // Nothing to restore: the commands build `target` of the volume, saved under the name
    // computed from the lock file once they succeed, before the step finishes.
    let first = fresh("step_first", "lock 1");
    let output = step_run(&first, false, "0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(
        &stdout,
        "not restored: api.melodium.tech has none of that name",
    );
    assert_contains(&stdout, "step: finished");
    assert!(!first.join("seen").exists());
    assert_eq!(completed(&mock), 1);
    assert_eq!(mock.state.lock().unwrap().caches.len(), 1);
    let name = mock
        .state
        .lock()
        .unwrap()
        .caches
        .keys()
        .next()
        .unwrap()
        .clone();
    assert!(name.starts_with("build-") && name.len() == 6 + 64, "{name}");
    assert_no_credentials(&output);

    // Restored before the commands run, on the local file system of the engine, the same
    // lock giving the same name.
    let second = fresh("step_second", "lock 1");
    let output = step_run(&second, true, "0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert!(!stdout.contains("not restored"), "{}", stdout);
    assert!(
        second.join("seen").exists(),
        "restored before the commands ran"
    );
    assert_eq!(completed(&mock), 2);

    // Another lock misses, and failing commands save nothing.
    let third = fresh("step_third", "lock 2");
    let output = step_run(&third, false, "1");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(
        &stdout,
        "not restored: api.melodium.tech has none of that name",
    );
    assert_contains(&stdout, "step: finished");
    assert!(!stdout.contains("step: success"), "{}", stdout);
    assert!(!stdout.contains("not saved"), "{}", stdout);
    assert_eq!(completed(&mock), 2);
    assert_eq!(mock.state.lock().unwrap().caches.len(), 1);

    for volume in [first, second, third] {
        let _ = std::fs::remove_dir_all(&volume);
    }
}

fn caches_need_api_melodium_tech() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let local = volume("local", &dependencies());

    let output = run(&save, None, &local, &["--name", "cargo"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "cache: not saved");
    assert_contains(
        &stdout,
        "api.melodium.tech is only reached by the programs it runs",
    );

    let output = run(
        &restore,
        None,
        &local,
        &["--name", "cargo", "--fallback", "cargo-old"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: missed");
    assert_contains(
        &stdout,
        "Cache `cargo` not restored: api.melodium.tech is only reached by the programs it runs",
    );

    let _ = std::fs::remove_dir_all(&local);
}

/// Paths leading out of the volume are refused, and the session opened meanwhile is aborted.
fn paths_stay_within_volumes() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let mock = api_server();
    let local = volume("paths", &dependencies());

    let output = run(
        &save,
        Some(&mock.address),
        &local,
        &["--name", "outside", "--path", "../.."],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "cache: not saved");
    assert_contains(&stdout, "a path led outside of the filesystem");
    {
        let state = mock.state.lock().unwrap();
        assert!(state.caches.is_empty());
        assert_eq!(state.aborted.len(), 1, "the session is aborted");
    }

    let output = run(
        &restore,
        Some(&mock.address),
        &local,
        &["--name", "cargo", "--fallback", "none", "--path", "."],
    );
    assert_contains(
        &String::from_utf8_lossy(&output.stdout),
        "it cannot be restored at the root of a file system",
    );
    assert!(local.join("target/CACHEDIR.TAG").exists());

    let _ = std::fs::remove_dir_all(&local);
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "caches_are_saved_then_restored",
        caches_are_saved_then_restored,
    ));
    cases.push((
        "identical_saves_keep_the_current_version",
        identical_saves_keep_the_current_version,
    ));
    cases.push(("misses_are_not_errors", misses_are_not_errors));
    cases.push(("full_storage_refuses_saves", full_storage_refuses_saves));
    cases.push((
        "tampered_archives_are_never_extracted",
        tampered_archives_are_never_extracted,
    ));
    // The program runs `sh`.
    #[cfg(unix)]
    cases.push((
        "revealed_values_never_leave_in_caches",
        revealed_values_never_leave_in_caches,
    ));
    cases.push((
        "run_tokens_never_leave_in_caches",
        run_tokens_never_leave_in_caches,
    ));
    // The step runs `sh`.
    #[cfg(unix)]
    cases.push((
        "steps_restore_before_and_save_after_commands",
        steps_restore_before_and_save_after_commands,
    ));
    cases.push((
        "caches_need_api_melodium_tech",
        caches_need_api_melodium_tech,
    ));
    cases.push(("paths_stay_within_volumes", paths_stay_within_volumes));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
