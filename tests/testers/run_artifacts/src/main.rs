//! Programs run by Cadence.CI push files and directories of their volumes as run artifacts,
//! with their access, retention, entrypoint and lock, and fetch artifacts of runs into their
//! volumes, each file checked against its sha256. Links are left out of artifacts, a file
//! holding a revealed secret is refused before anything is sent, a tampered or expired
//! artifact leaves nothing written, and steps of `cicd` push their artifacts once their
//! commands succeed.
//!
//! The API and storage are a mock.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use tester::{melodium, temp_file, temp_path};

const PUSH_SCRIPT: &str = include_str!("../../../run_artifacts_push.mel");
const FETCH_SCRIPT: &str = include_str!("../../../run_artifacts_fetch.mel");
const LEAK_SCRIPT: &str = include_str!("../../../run_artifacts_leak.mel");
const STEP_SCRIPT: &str = include_str!("../../../run_artifacts_step.mel");

const RUN_ID: &str = "15515515-0155-0155-0155-155155155155";
const RUN_TOKEN: &str = "cad_v_run-artifacts-run-token";
const SIGNATURE: &str = "presigned-signature-sentinel";
const PASSWORD: &str = "run-artifacts-password-sentinel";
/// Size of the parts of multipart uploads, below the size of the largest file pushed.
const PART_SIZE: usize = 16 * 1024;

/// Object in storage.
#[derive(Default)]
struct Object {
    content: Vec<u8>,
    parts: HashMap<u32, Vec<u8>>,
    multipart: bool,
}

/// File of an artifact.
#[derive(Clone, Debug)]
struct ArtifactFile {
    path: String,
    size: usize,
    media_type: String,
    object: String,
    sha256: Option<String>,
}

/// Artifact of the run, as pushed.
#[derive(Clone, Debug)]
struct Artifact {
    id: String,
    name: String,
    entrypoint: Option<String>,
    access: String,
    retention: String,
    lock: bool,
    files: Vec<ArtifactFile>,
    complete: bool,
}

#[derive(Default)]
struct State {
    /// Requests received, as method and path without the query, with their authorization.
    requests: Vec<(String, Option<String>)>,
    artifacts: Vec<Artifact>,
    objects: HashMap<String, Object>,
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
    body: Vec<u8>,
}

fn read_request(reader: &mut impl BufRead) -> Request {
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
    let mut length = 0;
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
            }
        }
    }
    let mut body = vec![0; length];
    let _ = reader.read_exact(&mut body);
    Request {
        method,
        path,
        query,
        authorization,
        body,
    }
}

/// Gives the first string value of `field` in `json`, as Mélodium sends it.
fn string(json: &str, field: &str) -> Option<String> {
    let (_, rest) = json.split_once(&format!("\"{field}\":\""))?;
    rest.split_once('"').map(|(value, _)| value.to_string())
}

/// Gives the number `field` in `json`.
fn number(json: &str, field: &str) -> usize {
    json.split_once(&format!("\"{field}\":"))
        .and_then(|(_, rest)| {
            rest.split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0)
}

/// Gives the objects of the array `field` in `json`.
fn objects(json: &str, field: &str) -> Vec<String> {
    let Some((_, rest)) = json.split_once(&format!("\"{field}\":[")) else {
        return Vec::new();
    };
    let mut objects = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let mut quoted = false;
    for (index, c) in rest.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '{' | '[' if !quoted => {
                if depth == 0 {
                    start = index;
                }
                depth += 1;
            }
            '}' | ']' if !quoted => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                if depth == 0 {
                    objects.push(rest[start..=index].to_string());
                }
            }
            _ => {}
        }
    }
    objects
}

fn presigned(address: &str, path: &str, method: &str) -> String {
    format!(
        r#"{{"url":"{address}{path}?X-Amz-Expires=900&X-Amz-Signature={SIGNATURE}","method":"{method}","headers":{{}}}}"#
    )
}

fn manifest(address: &str, files: &[ArtifactFile]) -> String {
    let files: Vec<String> = files
        .iter()
        .map(|file| {
            format!(
                r#"{{"path":"{}","size":{},"sha256":"{}","media_type":"{}","download":{}}}"#,
                file.path,
                file.size,
                file.sha256.clone().unwrap_or_default(),
                file.media_type,
                presigned(address, &format!("/storage/{}", file.object), "GET")
            )
        })
        .collect();
    // The artifact itself comes along, as `ArtifactDownload` of the API gives it.
    format!(
        r#"{{"artifact":{{"name":"listed","access":"organization","state":"complete"}},"files":[{}]}}"#,
        files.join(",")
    )
}

/// Starts a mock of the API keeping run artifacts, and of storage.
fn api_server() -> Mock {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let state = Arc::new(Mutex::new(State::default()));

    let shared = Arc::clone(&state);
    let base = address.clone();
    std::thread::spawn(move || {
        let mut next_id = 0;
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let request = read_request(&mut BufReader::new(stream.try_clone().unwrap()));
            let mut state = shared.lock().unwrap();
            state.requests.push((
                format!("{} {}", request.method, request.path),
                request.authorization.clone(),
            ));
            let json = String::from_utf8_lossy(&request.body).to_string();
            let run = format!("/execution/run/{RUN_ID}/artifacts");
            let authorized =
                request.authorization.as_deref() == Some(&format!("Bearer {RUN_TOKEN}"));
            let signed = request
                .query
                .contains(&format!("X-Amz-Signature={SIGNATURE}"));

            let (status, body): (&str, Vec<u8>) = match (
                request.method.as_str(),
                request.path.strip_prefix(&run),
                request.path.strip_prefix(&format!("/artifacts/{RUN_ID}/")),
                request.path.strip_prefix("/storage/"),
            ) {
                (_, Some(_), _, _) | (_, _, Some(_), _) if !authorized => {
                    ("401 Unauthorized", b"Run token required.".to_vec())
                }
                ("POST", Some(""), _, _) => {
                    let name = string(&json, "name").unwrap_or_default();
                    if state.artifacts.iter().any(|artifact| artifact.name == name) {
                        (
                            "409 Conflict",
                            format!("An artifact named `{name}` already exists in this run.")
                                .into_bytes(),
                        )
                    } else {
                        next_id += 1;
                        let id = format!("00000000-0000-0000-0001-{next_id:012}");
                        let mut files = Vec::new();
                        let mut uploads = Vec::new();
                        for (index, file) in objects(&json, "files").iter().enumerate() {
                            let path = string(file, "path").unwrap_or_default();
                            let size = number(file, "size");
                            let object = format!("{id}-{index}");
                            let multipart = size > PART_SIZE;
                            state.objects.insert(
                                object.clone(),
                                Object {
                                    multipart,
                                    ..Default::default()
                                },
                            );
                            let upload = if multipart {
                                let parts: Vec<String> = (1..=size.div_ceil(PART_SIZE))
                                    .map(|number| {
                                        presigned(
                                            &base,
                                            &format!("/storage/{object}/part/{number}"),
                                            "PUT",
                                        )
                                    })
                                    .collect();
                                format!(
                                    r#"{{"multipart":{{"part_size":{PART_SIZE},"parts":[{}]}}}}"#,
                                    parts.join(",")
                                )
                            } else {
                                format!(
                                    r#"{{"single":{{"request":{}}}}}"#,
                                    presigned(&base, &format!("/storage/{object}"), "PUT")
                                )
                            };
                            uploads.push(format!(r#"{{"path":"{path}","upload":{upload}}}"#));
                            files.push(ArtifactFile {
                                path,
                                size,
                                media_type: string(file, "media_type").unwrap_or_default(),
                                object,
                                sha256: None,
                            });
                        }
                        let retention = json
                            .split_once("\"retention\":")
                            .and_then(|(_, rest)| rest.split_once(",\"lock\""))
                            .map(|(retention, _)| retention.to_string())
                            .unwrap_or_default();
                        state.artifacts.push(Artifact {
                            id: id.clone(),
                            name,
                            entrypoint: string(&json, "entrypoint"),
                            access: string(&json, "access").unwrap_or_default(),
                            retention,
                            lock: json.contains("\"lock\":true"),
                            files,
                            complete: false,
                        });
                        (
                            "200 OK",
                            format!(
                                r#"{{"artifact_id":"{id}","uploads":[{}]}}"#,
                                uploads.join(",")
                            )
                            .into_bytes(),
                        )
                    }
                }
                ("POST", Some(complete), _, _) if complete.ends_with("/complete") => {
                    let id = complete
                        .trim_start_matches('/')
                        .trim_end_matches("/complete")
                        .to_string();
                    let completed = objects(&json, "files");
                    let artifact = state
                        .artifacts
                        .iter()
                        .position(|artifact| artifact.id == id)
                        .unwrap();
                    for file in completed {
                        let path = string(&file, "path").unwrap();
                        let index = state.artifacts[artifact]
                            .files
                            .iter()
                            .position(|known| known.path == path)
                            .unwrap();
                        let object = state.artifacts[artifact].files[index].object.clone();
                        let stored = state.objects.get_mut(&object).unwrap();
                        if stored.multipart {
                            assert_eq!(
                                file.matches("\"etag\"").count(),
                                stored.parts.len(),
                                "every part is completed"
                            );
                            assert_eq!(file.matches("\"part_number\"").count(), stored.parts.len());
                            let mut numbers: Vec<u32> = stored.parts.keys().copied().collect();
                            numbers.sort();
                            stored.content = numbers
                                .iter()
                                .flat_map(|number| stored.parts[number].clone())
                                .collect();
                        }
                        let size = stored.content.len();
                        let known = &mut state.artifacts[artifact].files[index];
                        assert_eq!(known.size, size, "the size announced is the size uploaded");
                        known.sha256 = string(&file, "sha256");
                    }
                    state.artifacts[artifact].complete = state.artifacts[artifact]
                        .files
                        .iter()
                        .all(|file| file.sha256.is_some());
                    ("200 OK", b"{}".to_vec())
                }
                ("GET", _, Some("expired"), _) => ("410 Gone", b"This artifact expired.".to_vec()),
                // An artifact whose files would reach out of the directory fetched into.
                ("GET", _, Some("evil"), _) => (
                    "200 OK",
                    manifest(
                        &base,
                        &[ArtifactFile {
                            path: "../escape".to_string(),
                            size: 4,
                            media_type: "text/plain".to_string(),
                            object: "none".to_string(),
                            sha256: Some("0".repeat(64)),
                        }],
                    )
                    .into_bytes(),
                ),
                ("GET", _, Some(name), _) if request.query == "manifest" => {
                    match state
                        .artifacts
                        .iter()
                        .find(|artifact| artifact.name == name && artifact.complete)
                    {
                        Some(artifact) => ("200 OK", manifest(&base, &artifact.files).into_bytes()),
                        None => (
                            "404 Not Found",
                            format!("No artifact named `{name}` in this run.").into_bytes(),
                        ),
                    }
                }
                (_, _, _, Some(_)) if !signed => (
                    "403 Forbidden",
                    b"<Error><Code>AccessDenied</Code></Error>".to_vec(),
                ),
                ("PUT", _, _, Some(target)) => match target.split_once("/part/") {
                    Some((object, number)) => {
                        let number: u32 = number.parse().unwrap();
                        state
                            .objects
                            .get_mut(object)
                            .unwrap()
                            .parts
                            .insert(number, request.body.clone());
                        ("200 OK", Vec::new())
                    }
                    None => {
                        state.objects.get_mut(target).unwrap().content = request.body.clone();
                        ("200 OK", Vec::new())
                    }
                },
                ("GET", _, _, Some(object)) => (
                    "200 OK",
                    state
                        .objects
                        .get(object)
                        .map(|object| object.content.clone())
                        .unwrap_or_default(),
                ),
                _ => ("404 Not Found", b"Not found.".to_vec()),
            };
            drop(state);

            let etag = if request.method == "PUT" && status.starts_with("200") {
                format!("ETag: \"etag-{}\"\r\n", request.path.len())
            } else {
                String::new()
            };
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\n{etag}Content-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(&body);
        }
    });

    Mock { address, state }
}

/// Fresh volume directory, holding the files given.
fn volume(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let path = temp_path("run_artifacts", name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    for (file, content) in files {
        let file = path.join(file);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    path
}

/// Files of a render, one of them larger than one part.
fn site() -> Vec<(&'static str, Vec<u8>)> {
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut bundle = Vec::with_capacity(40 * 1024);
    for _ in 0..40 * 1024 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        bundle.push(b'a' + (state % 26) as u8);
    }
    vec![
        (
            "site/index.html",
            b"<html><script src=\"assets/app.js\"></script></html>".to_vec(),
        ),
        ("site/assets/app.js", bundle),
        ("site/data.json", b"{\"tests\":42}".to_vec()),
    ]
}

/// Runs `script` with the volume `data` at `volume`, configured as on a Cadence.CI worker
/// when an API address is given, and as elsewhere otherwise.
fn run(script: &Path, api: Option<&str>, volume: &Path, args: &[&str]) -> Output {
    let mut command: Command = melodium();
    command
        .arg("run")
        .arg(script)
        .args(args)
        .env_remove("MELODIUM_API_URL")
        .env_remove("MELODIUM_API_TOKEN")
        .env_remove("MELODIUM_RUN_ID")
        .env("MELODIUM_RUN_VOLUMES", "data")
        .env("MELODIUM_RUN_VOLUME_data", volume)
        .env("MELODIUM_SECRET_ARTIFACT_PASSWORD", PASSWORD);
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

fn artifact(mock: &Mock, name: &str) -> Artifact {
    mock.state
        .lock()
        .unwrap()
        .artifacts
        .iter()
        .find(|artifact| artifact.name == name)
        .cloned()
        .unwrap()
}

fn pushed_trees_are_fetched_whole() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let mock = api_server();
    let pushed = volume("tree_pushed", &site());

    let output = run(
        &push,
        Some(&mock.address),
        &pushed,
        &["--source", "site", "--name", "report", "--access", "public"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(
        &stdout,
        &format!("address: {}/artifacts/{RUN_ID}/report", mock.address),
    );
    assert_no_credentials(&output);
    let report = artifact(&mock, "report");
    assert!(report.complete);
    assert_eq!(report.access, "public");
    assert_eq!(report.retention, "", "no retention for the default");
    assert!(!report.lock);
    assert_eq!(
        report
            .files
            .iter()
            .map(|file| (file.path.as_str(), file.media_type.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("assets/app.js", "text/javascript"),
            ("data.json", "application/json"),
            ("index.html", "text/html"),
        ]
    );
    {
        let state = mock.state.lock().unwrap();
        // Storage never gets the token of the run.
        assert!(state
            .requests
            .iter()
            .filter(|(target, _)| target.contains(" /storage/"))
            .all(|(_, authorization)| authorization.is_none()));
        assert!(state
            .requests
            .iter()
            .any(|(target, _)| target.contains("/part/3")));
    }

    let fetched = volume("tree_fetched", &[("fetched/kept.txt", b"kept".to_vec())]);
    let output = run(
        &fetch,
        Some(&mock.address),
        &fetched,
        &["--into", "fetched", "--run", RUN_ID, "--name", "report"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "artifact: fetched");
    assert_contains(
        &stdout,
        &format!(
            "Artifact 'report' of run {RUN_ID} fetched into 'fetched' of volume 'data', 3 files"
        ),
    );
    assert_no_credentials(&output);
    for (file, content) in site() {
        let fetched_file = fetched.join(file.replacen("site/", "fetched/", 1));
        assert_eq!(std::fs::read(&fetched_file).unwrap(), content, "{file}");
    }
    assert_eq!(
        std::fs::read(fetched.join("fetched/kept.txt")).unwrap(),
        b"kept"
    );

    // Names are unique within a run.
    let output = run(
        &push,
        Some(&mock.address),
        &pushed,
        &["--source", "site", "--name", "report"],
    );
    assert_contains(
        &String::from_utf8_lossy(&output.stdout),
        "Artifact 'report' not pushed from 'site' of volume 'data': Cadence.CI answered 409 Conflict: An artifact named `report` already exists in this run.",
    );

    let _ = std::fs::remove_dir_all(&pushed);
    let _ = std::fs::remove_dir_all(&fetched);
}

fn single_files_are_artifacts_of_one_file() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let mock = api_server();
    let pushed = volume(
        "single_pushed",
        &[("target/release/app", b"\x7fELF binary".to_vec())],
    );

    let output = run(
        &push,
        Some(&mock.address),
        &pushed,
        &["--source", "target/release/app", "--name", "app"],
    );
    assert!(output.status.success(), "{:?}", output);
    let app = artifact(&mock, "app");
    assert_eq!(app.files.len(), 1);
    assert_eq!(app.files[0].path, "app");
    assert_eq!(app.files[0].media_type, "application/octet-stream");
    assert_eq!(app.access, "organization");

    let fetched = volume("single_fetched", &[]);
    let output = run(
        &fetch,
        Some(&mock.address),
        &fetched,
        &["--into", "bin", "--run", RUN_ID, "--name", "app"],
    );
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(
        std::fs::read(fetched.join("bin/app")).unwrap(),
        b"\x7fELF binary"
    );

    let _ = std::fs::remove_dir_all(&pushed);
    let _ = std::fs::remove_dir_all(&fetched);
}

#[cfg(unix)]
fn links_are_left_out() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let mock = api_server();
    let pushed = volume("links", &site());
    std::os::unix::fs::symlink("/etc/passwd", pushed.join("site/passwd")).unwrap();

    let output = run(
        &push,
        Some(&mock.address),
        &pushed,
        &["--source", "site", "--name", "site"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(
        &stdout,
        "'passwd' is a link, left out of artifact 'site': artifacts hold files only",
    );
    assert!(artifact(&mock, "site")
        .files
        .iter()
        .all(|file| file.path != "passwd"));

    let _ = std::fs::remove_dir_all(&pushed);
}

fn retention_entrypoint_and_lock_reach_the_api() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let mock = api_server();
    let pushed = volume("retention", &site());

    let output = run(
        &push,
        Some(&mock.address),
        &pushed,
        &[
            "--source",
            "site",
            "--name",
            "render",
            "--retention",
            "30",
            "--entrypoint",
            "index.html",
        ],
    );
    assert!(output.status.success(), "{:?}", output);
    let render = artifact(&mock, "render");
    assert_eq!(render.retention, r#"{"days":30}"#);
    assert_eq!(render.entrypoint.as_deref(), Some("index.html"));

    let output = run(
        &push,
        Some(&mock.address),
        &pushed,
        &[
            "--source",
            "site",
            "--name",
            "release",
            "--expires",
            "false",
            "--lock",
            "true",
        ],
    );
    assert!(output.status.success(), "{:?}", output);
    let release = artifact(&mock, "release");
    assert_eq!(release.retention, "\"never\"");
    assert!(release.lock);

    for (args, expected) in [
        (
            vec!["--retention", "30", "--expires", "false"],
            "an artifact kept until deleted has no retention",
        ),
        (
            vec!["--entrypoint", "missing.html"],
            "its entrypoint 'missing.html' is not one of its files",
        ),
        (
            vec!["--access", "everyone"],
            "'everyone' is not an access, being 'organization' or 'public'",
        ),
    ] {
        let mut arguments = vec!["--source", "site", "--name", "refused"];
        arguments.extend(args);
        let output = run(&push, Some(&mock.address), &pushed, &arguments);
        assert_contains(&String::from_utf8_lossy(&output.stdout), expected);
    }
    assert!(mock
        .state
        .lock()
        .unwrap()
        .artifacts
        .iter()
        .all(|artifact| artifact.name != "refused"));

    let _ = std::fs::remove_dir_all(&pushed);
}

fn tampered_files_are_never_kept() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let mock = api_server();
    let pushed = volume("tampered_pushed", &site());

    let output = run(
        &push,
        Some(&mock.address),
        &pushed,
        &["--source", "site", "--name", "site"],
    );
    assert!(output.status.success(), "{:?}", output);
    // One file is altered in storage, its sha256 staying the one recorded.
    {
        let mut state = mock.state.lock().unwrap();
        let object = artifact_object(&state, "site", "data.json");
        state.objects.get_mut(&object).unwrap().content[2] ^= 0xff;
    }

    let fetched = volume("tampered_fetched", &[]);
    let output = run(
        &fetch,
        Some(&mock.address),
        &fetched,
        &["--into", "out/site", "--run", RUN_ID, "--name", "site"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(
        &stdout,
        &format!(
            "Artifact 'site' of run {RUN_ID} not fetched: 'data.json' not fetched, what storage gave does not match its sha256"
        ),
    );
    assert!(!stdout.contains("artifact: fetched"), "{}", stdout);
    assert_eq!(
        std::fs::read_dir(&fetched).unwrap().count(),
        0,
        "nothing written remains"
    );

    let _ = std::fs::remove_dir_all(&pushed);
    let _ = std::fs::remove_dir_all(&fetched);
}

fn artifact_object(state: &State, name: &str, path: &str) -> String {
    state
        .artifacts
        .iter()
        .find(|artifact| artifact.name == name)
        .unwrap()
        .files
        .iter()
        .find(|file| file.path == path)
        .unwrap()
        .object
        .clone()
}

fn expired_and_unsafe_artifacts_are_refused() {
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let mock = api_server();
    let fetched = volume("refused", &[]);

    for (name, expected) in [
        ("expired", "it expired".to_string()),
        (
            "evil",
            "'../escape' is not a path within an artifact".to_string(),
        ),
        (
            "unknown",
            "Cadence.CI answered 404 Not Found: No artifact named `unknown` in this run."
                .to_string(),
        ),
    ] {
        let output = run(
            &fetch,
            Some(&mock.address),
            &fetched,
            &["--into", "out", "--run", RUN_ID, "--name", name],
        );
        assert_contains(
            &String::from_utf8_lossy(&output.stdout),
            &format!("Artifact '{name}' of run {RUN_ID} not fetched: {expected}"),
        );
    }
    assert_eq!(std::fs::read_dir(&fetched).unwrap().count(), 0);
    assert!(!fetched.parent().unwrap().join("escape").exists());

    let _ = std::fs::remove_dir_all(&fetched);
}

#[cfg(unix)]
fn revealed_values_never_leave_in_artifacts() {
    let leak = temp_file("run_artifacts", "leak.mel", LEAK_SCRIPT);
    let mock = api_server();
    let leaking = volume("leak", &site());

    let output = run(
        &leak,
        Some(&mock.address),
        &leaking,
        &["--directory", leaking.join("site").to_str().unwrap()],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        std::fs::read_to_string(leaking.join("site/config"))
            .unwrap()
            .contains(PASSWORD),
        "the command got the password"
    );
    assert_contains(
        &stdout,
        "Artifact 'site' not pushed from 'site' of volume 'data': 'config' holds the value of secret",
    );
    assert!(!stdout.contains("artifact: pushed"), "{}", stdout);
    // Nothing was sent, not even the list of files.
    assert!(mock.state.lock().unwrap().requests.is_empty());
    assert_no_credentials(&output);

    let _ = std::fs::remove_dir_all(&leaking);
}

#[cfg(unix)]
fn steps_push_artifacts_once_commands_succeed() {
    let step = temp_file("run_artifacts", "step.mel", STEP_SCRIPT);
    let mock = api_server();

    let failing = volume("step_failing", &[]);
    let output = run(
        &step,
        Some(&mock.address),
        &failing,
        &["--directory", failing.to_str().unwrap(), "--exit", "1"],
    );
    assert_contains(&String::from_utf8_lossy(&output.stdout), "step: finished");
    assert!(mock.state.lock().unwrap().artifacts.is_empty());

    let succeeding = volume("step_succeeding", &[]);
    let output = run(
        &step,
        Some(&mock.address),
        &succeeding,
        &["--directory", succeeding.to_str().unwrap()],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    let pushed = stdout
        .find(&format!(
            "Artifact 'out' pushed, at {}/artifacts/{RUN_ID}/out",
            mock.address
        ))
        .expect(&stdout);
    assert!(
        pushed < stdout.find("step: finished").expect(&stdout),
        "{}",
        stdout
    );
    let out = artifact(&mock, "out");
    assert!(out.complete);
    assert_eq!(out.access, "public");

    let _ = std::fs::remove_dir_all(&failing);
    let _ = std::fs::remove_dir_all(&succeeding);
}

fn artifacts_need_cadence_ci() {
    let push = temp_file("run_artifacts", "push.mel", PUSH_SCRIPT);
    let fetch = temp_file("run_artifacts", "fetch.mel", FETCH_SCRIPT);
    let local = volume("local", &site());

    let output = run(&push, None, &local, &["--source", "site", "--name", "site"]);
    assert_contains(
        &String::from_utf8_lossy(&output.stdout),
        "Artifact 'site' not pushed from 'site' of volume 'data': artifacts are only pushed by programs run by Cadence.CI, and MELODIUM_API_URL, MELODIUM_API_TOKEN, MELODIUM_RUN_ID are not set",
    );
    let output = run(
        &fetch,
        None,
        &local,
        &["--into", "out", "--run", RUN_ID, "--name", "site"],
    );
    assert_contains(
        &String::from_utf8_lossy(&output.stdout),
        "artifacts are only fetched by programs run by Cadence.CI, and MELODIUM_API_URL, MELODIUM_API_TOKEN, MELODIUM_RUN_ID are not set",
    );

    let mock = api_server();
    let output = run(
        &push,
        Some(&mock.address),
        &local,
        &["--source", "site", "--name", "my report"],
    );
    assert_contains(
        &String::from_utf8_lossy(&output.stdout),
        "'my report' is not an artifact name, made of letters, digits, '_', '.' and '-'",
    );
    let output = run(
        &push,
        Some(&mock.address),
        &local,
        &["--source", "../..", "--name", "outside"],
    );
    assert_contains(
        &String::from_utf8_lossy(&output.stdout),
        "'../..' is not a path within volume 'data', relative and without '..'",
    );
    assert!(mock.state.lock().unwrap().requests.is_empty());

    let _ = std::fs::remove_dir_all(&local);
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "pushed_trees_are_fetched_whole",
        pushed_trees_are_fetched_whole,
    ));
    cases.push((
        "single_files_are_artifacts_of_one_file",
        single_files_are_artifacts_of_one_file,
    ));
    #[cfg(unix)]
    cases.push(("links_are_left_out", links_are_left_out));
    cases.push((
        "retention_entrypoint_and_lock_reach_the_api",
        retention_entrypoint_and_lock_reach_the_api,
    ));
    cases.push((
        "tampered_files_are_never_kept",
        tampered_files_are_never_kept,
    ));
    cases.push((
        "expired_and_unsafe_artifacts_are_refused",
        expired_and_unsafe_artifacts_are_refused,
    ));
    // The programs run `sh`.
    #[cfg(unix)]
    cases.push((
        "revealed_values_never_leave_in_artifacts",
        revealed_values_never_leave_in_artifacts,
    ));
    #[cfg(unix)]
    cases.push((
        "steps_push_artifacts_once_commands_succeed",
        steps_push_artifacts_once_commands_succeed,
    ));
    cases.push(("artifacts_need_cadence_ci", artifacts_need_cadence_ci));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
