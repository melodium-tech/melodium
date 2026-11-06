//! Programs run by api.melodium.tech push files of directories, and single files, as run
//! artifacts, and fetch them back, assembled in Mélodium (`work/artifact`): each file in a track
//! of its own, read, searched for revealed secret values with its path, and uploaded in parts
//! while it is read; fetched files are downloaded with resumption and checked against their
//! sha256, a file that differs being removed. Steps push their artifacts once their commands
//! succeed. No presigned URL or token appears in what runs show.
//!
//! The API and storage are a mock following the contract of api.melodium.tech (artifacts whose
//! files go through upload sessions), refusing a part once and cutting a download once.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use tester::{melodium, temp_file, temp_path};

const PUSH_SCRIPT: &str = include_str!("../../../run_artifacts_push.mel");
const FILE_SCRIPT: &str = include_str!("../../../run_artifacts_file.mel");
const FETCH_SCRIPT: &str = include_str!("../../../run_artifacts_fetch.mel");
const STEP_SCRIPT: &str = include_str!("../../../run_artifacts_step.mel");

const RUN_ID: &str = "15515515-0155-0155-0155-155155155155";
const RUN_TOKEN: &str = "cad_v_run-artifacts-run-token";
const SIGNATURE: &str = "presigned-signature-sentinel";
const MIB: u64 = 1024 * 1024;

/// File of an artifact, as stored.
#[derive(Clone)]
struct File {
    sha256: String,
    object: String,
}

/// Artifact of the run.
#[derive(Clone, Default)]
struct Artifact {
    request: Value,
    complete: bool,
    opened: Vec<String>,
    files: std::collections::BTreeMap<String, File>,
}

/// Upload session.
#[derive(Default)]
struct Upload {
    artifact_id: String,
    path: String,
    presigned: Vec<u64>,
    parts: HashMap<u64, Vec<u8>>,
    tags: HashMap<u64, String>,
}

#[derive(Default)]
struct State {
    /// Requests received, as method and path without the query, with their authorization.
    requests: Vec<(String, Option<String>)>,
    artifacts: HashMap<String, Artifact>,
    objects: HashMap<String, Vec<u8>>,
    uploads: HashMap<String, Upload>,
    aborted: Vec<String>,
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
    let authorized = request.authorization.as_deref() == Some(&format!("Bearer {RUN_TOKEN}"));
    // Stable addresses, read by runs of the organization.
    if let Some(address) = request.path.strip_prefix("/artifacts/") {
        let (run, name) = address.split_once('/').unwrap_or_default();
        let found = state
            .artifacts
            .values()
            .find(|artifact| artifact.complete && artifact.request["name"] == name);
        return Some(match found {
            Some(artifact) if authorized && run == RUN_ID && request.query == "manifest" => {
                let files: Vec<Value> = artifact
                    .files
                    .iter()
                    .map(|(path, file)| {
                        json!({
                            "path": path,
                            "size": state.objects[&file.object].len(),
                            "sha256": file.sha256,
                            "media_type": "application/octet-stream",
                            "download": presigned(base, &format!("/storage/{}", file.object), "GET"),
                        })
                    })
                    .collect();
                (200, json!({"artifact": {"name": name}, "files": files}))
            }
            _ => (404, json!({"error": "The artifact does not exist"})),
        });
    }
    let path = request
        .path
        .strip_prefix(&format!("/execution/run/{RUN_ID}/"))?;
    if !authorized {
        return Some((403, json!({"error": "not this run's machine token"})));
    }
    let body: Value = serde_json::from_slice(&request.body).unwrap_or(Value::Null);
    let segments: Vec<&str> = path.split('/').collect();
    Some(match (request.method.as_str(), segments.as_slice()) {
        ("POST", ["artifacts"]) => {
            *next += 1;
            let artifact_id = format!("00000000-0000-0000-0001-{:012}", *next);
            state.artifacts.insert(
                artifact_id.clone(),
                Artifact {
                    request: body,
                    ..Default::default()
                },
            );
            (200, json!({"artifact_id": artifact_id}))
        }
        ("POST", ["artifacts", artifact_id, "complete"]) => {
            let Some(artifact) = state.artifacts.get_mut(*artifact_id) else {
                return Some((403, json!({"error": "not an artifact of this run"})));
            };
            if artifact.files.is_empty() || artifact.files.len() != artifact.opened.len() {
                return Some((400, json!({"error": "no file, or a file not completed"})));
            }
            artifact.complete = true;
            (
                200,
                json!({
                    "id": artifact_id,
                    "name": artifact.request["name"],
                    "state": "complete",
                    "files": artifact.files.keys().collect::<Vec<_>>(),
                }),
            )
        }
        ("POST", ["storage", "uploads"]) => {
            let file = &body["purpose"]["artifact_file"];
            let artifact_id = file["artifact_id"].as_str().unwrap_or_default();
            let path = file["path"].as_str().unwrap_or_default();
            let Some(artifact) = state.artifacts.get_mut(artifact_id) else {
                return Some((403, json!({"error": "not an artifact of this run"})));
            };
            if artifact.opened.iter().any(|opened| opened == path) {
                return Some((409, json!({"error": "path already opened"})));
            }
            artifact.opened.push(path.to_string());
            *next += 1;
            let upload_id = format!("00000000-0000-0000-0000-{:012}", *next);
            state.uploads.insert(
                upload_id.clone(),
                Upload {
                    artifact_id: artifact_id.to_string(),
                    path: path.to_string(),
                    ..Default::default()
                },
            );
            (200, json!({"upload_id": upload_id, "upload": "parts"}))
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
            state.objects.insert(upload_id.to_string(), content);
            state
                .artifacts
                .get_mut(&upload.artifact_id)
                .unwrap()
                .files
                .insert(
                    upload.path.clone(),
                    File {
                        sha256,
                        object: upload_id.to_string(),
                    },
                );
            (
                200,
                json!({"artifact_file": {"artifact_id": upload.artifact_id, "path": upload.path}}),
            )
        }
        ("POST", ["storage", "uploads", upload_id, "abort"]) => {
            state.uploads.remove(*upload_id);
            state.aborted.push(upload_id.to_string());
            (200, json!({"success": true}))
        }
        _ => (404, json!({"error": "not found"})),
    })
}

/// Starts a mock of the API giving artifacts, and of storage.
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

/// Fresh volume directory, holding `site` with the files given.
fn volume(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let path = temp_path("run_artifacts", name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(path.join("site")).unwrap();
    for (file, content) in files {
        let file = path.join("site").join(file);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    path
}

/// Files of a render, one of them needing two parts.
fn site() -> Vec<(&'static str, Vec<u8>)> {
    // Barely compressible content.
    let mut state: u64 = 0x2545F4914F6CDD1D;
    let mut noise = Vec::with_capacity(12 * MIB as usize);
    for _ in 0..12 * MIB {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        noise.push(state as u8);
    }
    vec![
        ("index.html", b"<h1>Report</h1>".to_vec()),
        ("assets/css/main.css", b"h1 { color: teal; }".to_vec()),
        ("assets/data.bin", noise),
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
        .env("MELODIUM_RUN_VOLUME_data", volume);
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
        for credential in [SIGNATURE, RUN_TOKEN] {
            assert!(!text.contains(credential), "{credential} shown in:\n{text}");
        }
    }
}

/// The artifact named `name`, once complete.
fn artifact(mock: &Mock, name: &str) -> Option<Artifact> {
    mock.state
        .lock()
        .unwrap()
        .artifacts
        .values()
        .find(|artifact| artifact.complete && artifact.request["name"] == name)
        .cloned()
}

fn directories_are_pushed_then_fetched() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let mock = api_server();
    let pushed = volume("pushed", &site());
    #[cfg(unix)]
    std::os::unix::fs::symlink("index.html", pushed.join("site/home.html")).unwrap();

    let output = run(&push, Some(&mock.address), &pushed, &["--name", "report"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "artifact: pushed");
    assert_contains(&stdout, r#""state":"complete""#);
    assert_no_credentials(&output);
    let report = artifact(&mock, "report").expect(&stdout);
    assert_eq!(
        report.request,
        json!({
            "name": "report",
            "entrypoint": "index.html",
            "access": "public",
            "retention": {"days": 30},
            "lock": false,
        })
    );
    // Links are left out.
    assert_eq!(
        report.files.keys().collect::<Vec<_>>(),
        ["assets/css/main.css", "assets/data.bin", "index.html"]
    );
    {
        let state = mock.state.lock().unwrap();
        assert_eq!(
            state.refused_parts.len(),
            1,
            "a part was refused once, then sent again"
        );
        assert!(state
            .requests
            .iter()
            .filter(|(target, _)| target.contains(" /storage/"))
            .all(|(_, authorization)| authorization.is_none()));
    }

    // Fetched back by another run of the organization, a download being cut once.
    let output = run(
        &fetch,
        Some(&mock.address),
        &pushed,
        &["--run", RUN_ID, "--name", "report"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "artifact: fetched");
    assert_no_credentials(&output);
    for (file, content) in site() {
        assert_eq!(
            std::fs::read(pushed.join("fetched").join(file)).unwrap(),
            content,
            "{file}"
        );
    }
    assert!(!pushed.join("fetched/home.html").exists());
    assert!(!mock.state.lock().unwrap().cut_downloads.is_empty());

    let _ = std::fs::remove_dir_all(&pushed);
}

fn files_are_pushed_alone() {
    let file = temp_file("run_artifacts", "file.mel", FILE_SCRIPT);
    let mock = api_server();
    let pushed = volume("alone", &[("bin/hexyl", b"\x7fELF binary".to_vec())]);

    let output = run(
        &file,
        Some(&mock.address),
        &pushed,
        &["--name", "binary", "--path", "site/bin/hexyl"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "artifact: pushed");
    let binary = artifact(&mock, "binary").expect(&stdout);
    assert_eq!(binary.request["retention"], json!("never"));
    assert_eq!(binary.request["entrypoint"], Value::Null);
    assert_eq!(binary.files.keys().collect::<Vec<_>>(), ["hexyl"]);

    let _ = std::fs::remove_dir_all(&pushed);
}

fn tampered_files_are_removed() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let mock = api_server();
    let pushed = volume("tampered", &site());

    let output = run(&push, Some(&mock.address), &pushed, &["--name", "report"]);
    assert!(output.status.success(), "{:?}", output);
    // A file is altered in storage, its sha256 staying the one recorded.
    {
        let mut state = mock.state.lock().unwrap();
        let object = state
            .artifacts
            .values()
            .find(|artifact| artifact.complete)
            .unwrap()
            .files["index.html"]
            .object
            .clone();
        state.objects.get_mut(&object).unwrap()[0] ^= 0xff;
    }

    let output = run(
        &fetch,
        Some(&mock.address),
        &pushed,
        &["--run", RUN_ID, "--name", "report"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "artifact: not fetched");
    assert_contains(
        &stdout,
        "'index.html' is not the file api.melodium.tech recorded",
    );
    assert!(!pushed.join("fetched/index.html").exists());
    assert_no_credentials(&output);

    let _ = std::fs::remove_dir_all(&pushed);
}

/// The token of the run is masked from the start: a file holding it, or a path naming it, is
/// never uploaded whole, and the artifact never completes.
fn revealed_values_never_leave_in_artifacts() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let mock = api_server();
    let content = volume(
        "leak_content",
        &[
            ("index.html", b"index".to_vec()),
            ("env", format!("MELODIUM_API_TOKEN={RUN_TOKEN}\n").into_bytes()),
        ],
    );

    let output = run(&push, Some(&mock.address), &content, &["--name", "leak"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "artifact: not pushed");
    assert_contains(
        &stdout,
        "the data holds a value revealed from secret 'api-run:token'",
    );
    assert!(artifact(&mock, "leak").is_none());
    assert_no_credentials(&output);

    let path = volume(
        "leak_path",
        &[(&format!("{RUN_TOKEN}.txt"), b"named after the token".to_vec())],
    );
    let output = run(&push, Some(&mock.address), &path, &["--name", "named"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "artifact: not pushed");
    assert_contains(
        &stdout,
        "the data holds a value revealed from secret 'api-run:token'",
    );
    assert!(artifact(&mock, "named").is_none());

    let _ = std::fs::remove_dir_all(&content);
    let _ = std::fs::remove_dir_all(&path);
}

fn missing_artifacts_fail_fetches() {
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let mock = api_server();
    let empty = volume("missing", &[]);

    let output = run(
        &fetch,
        Some(&mock.address),
        &empty,
        &["--run", RUN_ID, "--name", "nothing"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "artifact: not fetched");
    assert_contains(
        &stdout,
        "api.melodium.tech answered 404 Not Found: {\"error\":\"The artifact does not exist\"}",
    );
    assert!(!empty.join("fetched").exists());

    let _ = std::fs::remove_dir_all(&empty);
}

fn artifacts_need_api_melodium_tech() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let local = volume("local", &site());

    for output in [
        run(&push, None, &local, &["--name", "report"]),
        run(&fetch, None, &local, &["--run", RUN_ID, "--name", "report"]),
    ] {
        assert_contains(
            &String::from_utf8_lossy(&output.stdout),
            "api.melodium.tech is only reached by the programs it runs",
        );
    }

    let _ = std::fs::remove_dir_all(&local);
}

#[cfg(unix)]
fn steps_push_artifacts_once_commands_succeed() {
    let step = temp_file("run_artifacts", "step.mel", STEP_SCRIPT);
    let mock = api_server();

    // A directory of the volume and a file of the local file system, pushed before the step
    // finishes.
    let succeeding = volume("step_success", &[]);
    let output = run(
        &step,
        Some(&mock.address),
        &succeeding,
        &["--directory", succeeding.to_str().unwrap(), "--exit", "0"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "step: finished");
    assert!(!stdout.contains("not pushed"), "{}", stdout);
    let report = artifact(&mock, "report").expect(&stdout);
    assert_eq!(
        report.files.keys().collect::<Vec<_>>(),
        ["index.html", "nested/data.txt"]
    );
    let binary = artifact(&mock, "binary").expect(&stdout);
    assert_eq!(binary.files.keys().collect::<Vec<_>>(), ["tool"]);
    assert_no_credentials(&output);

    // Failing commands push nothing.
    let failing = volume("step_failure", &[]);
    let output = run(
        &step,
        Some(&mock.address),
        &failing,
        &["--directory", failing.to_str().unwrap(), "--exit", "1"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "step: finished");
    assert!(!stdout.contains("step: success"), "{}", stdout);
    assert_eq!(mock.state.lock().unwrap().artifacts.len(), 2);

    // Outside api.melodium.tech, the step succeeds, only warning.
    let local = volume("step_local", &[]);
    let output = run(
        &step,
        None,
        &local,
        &["--directory", local.to_str().unwrap(), "--exit", "0"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, "step: success");
    assert_contains(&stdout, "step: finished");
    assert_contains(
        &stdout,
        "Artifact `report` not pushed: api.melodium.tech is only reached by the programs it runs",
    );

    for volume in [succeeding, failing, local] {
        let _ = std::fs::remove_dir_all(&volume);
    }
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "directories_are_pushed_then_fetched",
        directories_are_pushed_then_fetched,
    ));
    cases.push(("files_are_pushed_alone", files_are_pushed_alone));
    cases.push(("tampered_files_are_removed", tampered_files_are_removed));
    cases.push((
        "revealed_values_never_leave_in_artifacts",
        revealed_values_never_leave_in_artifacts,
    ));
    cases.push((
        "missing_artifacts_fail_fetches",
        missing_artifacts_fail_fetches,
    ));
    cases.push((
        "artifacts_need_api_melodium_tech",
        artifacts_need_api_melodium_tech,
    ));
    // The step runs `sh`.
    #[cfg(unix)]
    cases.push((
        "steps_push_artifacts_once_commands_succeed",
        steps_push_artifacts_once_commands_succeed,
    ));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
