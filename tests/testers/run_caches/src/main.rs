//! Programs run by Cadence.CI save directories of their volumes as caches, and later runs
//! restore them, by exact key or by prefix. Archives go to storage through presigned
//! requests, in parts with retries, and come back with resumed downloads checked against
//! their sha256. A miss is never an error, a tampered archive is never extracted, a cache
//! holding a revealed secret is refused, and no presigned URL appears in what runs show.
//!
//! The API and storage are a mock, uploads failing once and downloads being cut once.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
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
/// Size of the parts of multipart uploads, below the size of the caches saved.
const PART_SIZE: usize = 16 * 1024;

/// Object in storage.
#[derive(Clone, Default)]
struct Object {
    /// Content of a single upload, then of the whole object once complete.
    content: Vec<u8>,
    /// Parts of a multipart upload, by number.
    parts: HashMap<u32, Vec<u8>>,
    multipart: bool,
}

/// Cache of the organization.
#[derive(Clone)]
struct Cache {
    key: String,
    object: String,
    size: usize,
    sha256: Option<String>,
}

#[derive(Default)]
struct State {
    /// Requests received, as method and path without the query, with their authorization.
    requests: Vec<(String, Option<String>)>,
    caches: Vec<Cache>,
    objects: HashMap<String, Object>,
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
    let mut range = None;
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
            } else if name.eq_ignore_ascii_case("range") {
                range = value
                    .strip_prefix("bytes=")
                    .and_then(|range| range.strip_suffix('-'))
                    .and_then(|start| start.parse().ok());
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
        range,
        body,
    }
}

/// Gives the string values of `field` in `json`, a flat JSON object as Mélodium sends them.
fn strings(json: &str, field: &str) -> Vec<String> {
    let Some((_, rest)) = json.split_once(&format!("\"{field}\":")) else {
        return Vec::new();
    };
    let rest = rest.trim_start();
    let values = if let Some(array) = rest.strip_prefix('[') {
        array
            .split_once(']')
            .map(|(values, _)| values)
            .unwrap_or_default()
    } else {
        rest.split([',', '}']).next().unwrap_or_default()
    };
    values
        .split(',')
        .filter_map(|value| {
            value
                .trim()
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
        .map(str::to_string)
        .collect()
}

/// Gives the number `field` in `json`.
fn number(json: &str, field: &str) -> usize {
    json.split_once(&format!("\"{field}\":"))
        .and_then(|(_, rest)| {
            rest.trim_start()
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0)
}

fn presigned(address: &str, path: &str, method: &str) -> String {
    format!(
        r#"{{"url":"{address}{path}?X-Amz-Expires=900&X-Amz-Signature={SIGNATURE}","method":"{method}","headers":{{"x-amz-meta-run":"{RUN_ID}"}}}}"#
    )
}

/// Starts a mock of the API giving caches, and of storage.
fn api_server() -> Mock {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let state = Arc::new(Mutex::new(State::default()));

    let shared = Arc::clone(&state);
    let base = address.clone();
    std::thread::spawn(move || {
        let mut next_object = 0;
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let request = read_request(&mut BufReader::new(stream.try_clone().unwrap()));
            let mut state = shared.lock().unwrap();
            state.requests.push((
                format!("{} {}", request.method, request.path),
                request.authorization.clone(),
            ));
            let json = String::from_utf8_lossy(&request.body).to_string();
            let run = format!("/execution/run/{RUN_ID}/cache/");
            let authorized =
                request.authorization.as_deref() == Some(&format!("Bearer {RUN_TOKEN}"));
            let signed = request
                .query
                .contains(&format!("X-Amz-Signature={SIGNATURE}"));

            let mut cut = false;
            let (status, body): (&str, Vec<u8>) = match (
                request.method.as_str(),
                request.path.strip_prefix(&run),
                request.path.strip_prefix("/storage/"),
            ) {
                (_, Some(_), _) if !authorized => {
                    ("401 Unauthorized", b"Run token required.".to_vec())
                }
                ("POST", Some("restore"), _) => {
                    let complete: Vec<&Cache> = state
                        .caches
                        .iter()
                        .filter(|cache| cache.sha256.is_some())
                        .collect();
                    let found = strings(&json, "keys").iter().find_map(|key| {
                        complete
                            .iter()
                            .find(|cache| &cache.key == key)
                            .or_else(|| {
                                complete
                                    .iter()
                                    .rev()
                                    .find(|cache| cache.key.starts_with(key.as_str()))
                            })
                            .map(|cache| (*cache).clone())
                    });
                    match found {
                        Some(cache) => (
                            "200 OK",
                            format!(
                                r#"{{"hit":{{"key":"{}","size":{},"sha256":"{}","download":{}}}}}"#,
                                cache.key,
                                cache.size,
                                cache.sha256.unwrap(),
                                presigned(&base, &format!("/storage/{}", cache.object), "GET")
                            )
                            .into_bytes(),
                        ),
                        None => ("200 OK", br#""miss""#.to_vec()),
                    }
                }
                ("POST", Some("save"), _) => {
                    let key = strings(&json, "key").pop().unwrap_or_default();
                    let size = number(&json, "size");
                    if state
                        .caches
                        .iter()
                        .any(|cache| cache.key == key && cache.sha256.is_some())
                    {
                        ("200 OK", br#""exists""#.to_vec())
                    } else if key.starts_with("full-") {
                        (
                            "200 OK",
                            br#"{"skipped":{"reason":"storage full"}}"#.to_vec(),
                        )
                    } else {
                        next_object += 1;
                        let object = format!("00000000-0000-0000-0000-{next_object:012}");
                        let multipart = size > PART_SIZE;
                        state.objects.insert(
                            object.clone(),
                            Object {
                                multipart,
                                ..Default::default()
                            },
                        );
                        state.caches.push(Cache {
                            key,
                            object: object.clone(),
                            size,
                            sha256: None,
                        });
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
                        (
                            "200 OK",
                            format!(r#"{{"upload":{{"object_id":"{object}","upload":{upload}}}}}"#)
                                .into_bytes(),
                        )
                    }
                }
                ("POST", Some(complete), _)
                    if complete.starts_with("save/") && complete.ends_with("/complete") =>
                {
                    let object = complete
                        .trim_start_matches("save/")
                        .trim_end_matches("/complete")
                        .to_string();
                    let sha256 = strings(&json, "sha256").pop();
                    let etags = json.matches("\"etag\"").count();
                    assert_eq!(json.matches("\"part_number\"").count(), etags);
                    let stored = state.objects.get_mut(&object).unwrap();
                    if stored.multipart {
                        assert_eq!(etags, stored.parts.len(), "every part is completed");
                        let mut numbers: Vec<u32> = stored.parts.keys().copied().collect();
                        numbers.sort();
                        stored.content = numbers
                            .iter()
                            .flat_map(|number| stored.parts[number].clone())
                            .collect();
                    }
                    let size = stored.content.len();
                    let cache = state
                        .caches
                        .iter_mut()
                        .find(|cache| cache.object == object)
                        .unwrap();
                    assert_eq!(cache.size, size, "the size reserved is the size uploaded");
                    cache.sha256 = sha256;
                    ("200 OK", b"{}".to_vec())
                }
                (_, _, Some(_)) if !signed => (
                    "403 Forbidden",
                    b"<Error><Code>AccessDenied</Code></Error>".to_vec(),
                ),
                ("PUT", _, Some(target)) => {
                    let label = target.to_string();
                    match target.split_once("/part/") {
                        // The first upload of the second part fails, then succeeds.
                        Some((_, "2")) if !state.refused_parts.contains(&label) => {
                            state.refused_parts.push(label);
                            (
                                "503 Service Unavailable",
                                b"<Error><Code>SlowDown</Code></Error>".to_vec(),
                            )
                        }
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
                    }
                }
                ("GET", _, Some(object)) => {
                    let content = state
                        .objects
                        .get(object)
                        .map(|object| object.content.clone())
                        .unwrap_or_default();
                    let start = request.range.unwrap_or(0);
                    // The first download of each object is cut halfway.
                    if !state.cut_downloads.contains(&object.to_string())
                        && start == 0
                        && content.len() > 1
                    {
                        state.cut_downloads.push(object.to_string());
                        cut = true;
                    }
                    if start > 0 {
                        let _ = write!(
                            stream,
                            "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {start}-{}/{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            content.len() - 1,
                            content.len(),
                            content.len() - start
                        );
                        let _ = stream.write_all(&content[start..]);
                        continue;
                    }
                    ("200 OK", content)
                }
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
            if cut {
                let _ = stream.write_all(&body[..body.len() / 2]);
                let _ = stream.flush();
            } else {
                let _ = stream.write_all(&body);
            }
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

/// Files of a dependency cache, larger than one part once archived.
fn dependencies() -> Vec<(&'static str, Vec<u8>)> {
    // Barely compressible content, so that the archive needs several parts.
    let mut state: u64 = 0x2545F4914F6CDD1D;
    let mut noise = Vec::with_capacity(64 * 1024);
    for _ in 0..64 * 1024 {
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

/// Runs `script` with the volume `data` at `volume`, configured as on a Cadence.CI worker
/// when an API address is given, and as elsewhere otherwise.
fn run(script: &Path, api: Option<&str>, volume: &Path, args: &[&str]) -> Output {
    let mut command: Command = melodium();
    command.arg("run");
    // `keys` is a vector, given as a literal.
    if args.contains(&"--keys") {
        command.arg("--parse-arguments");
    }
    command
        .arg(script)
        .args(args)
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

fn caches_are_saved_then_restored() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let mock = api_server();
    let saved = volume("saved", &dependencies());

    let output = run(&save, Some(&mock.address), &saved, &["--key", "cargo-abc"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: saved");
    assert_contains(
        &stdout,
        "Cache 'cargo-abc' saved from 'target' of volume 'data'",
    );
    assert_no_credentials(&output);
    {
        let state = mock.state.lock().unwrap();
        assert!(state.caches[0].sha256.is_some());
        assert_eq!(
            state.refused_parts.len(),
            1,
            "a part was refused once, then sent again"
        );
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

    // A later run restores it by prefix, the exact key missing.
    let restored = volume("restored", &[("kept.txt", b"kept".to_vec())]);
    let output = run(
        &restore,
        Some(&mock.address),
        &restored,
        &["--keys", r#"["cargo-zzz", "cargo-"]"#],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "matched: cargo-abc");
    assert_no_credentials(&output);
    assert_same_files(&saved, &restored.join("target"));
    assert_eq!(
        std::fs::read(restored.join("target/kept.txt")).unwrap(),
        b"kept"
    );
    assert_eq!(
        mock.state.lock().unwrap().cut_downloads.len(),
        1,
        "the download was cut once, then resumed"
    );

    let _ = std::fs::remove_dir_all(&saved);
    let _ = std::fs::remove_dir_all(&restored);
}

fn existing_keys_upload_nothing() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let mock = api_server();
    let first = volume("first", &dependencies());
    let second = volume("second", &[("other.txt", b"other".to_vec())]);

    let output = run(&save, Some(&mock.address), &first, &["--key", "cargo-abc"]);
    assert!(output.status.success(), "{:?}", output);
    let uploads = mock.state.lock().unwrap().requests.len();

    let output = run(&save, Some(&mock.address), &second, &["--key", "cargo-abc"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(
        &stdout,
        "Cache 'cargo-abc' already exists, nothing uploaded",
    );
    assert_contains(&stdout, "cache: saved");
    let state = mock.state.lock().unwrap();
    assert_eq!(
        state.requests[uploads..]
            .iter()
            .map(|(target, _)| target.as_str())
            .collect::<Vec<_>>(),
        vec![format!("POST /execution/run/{RUN_ID}/cache/save").as_str()]
    );

    let _ = std::fs::remove_dir_all(&first);
    let _ = std::fs::remove_dir_all(&second);
}

/// Cadence.CI skips caches it has no room for: nothing is uploaded, and the run goes on.
fn skipped_caches_upload_nothing() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let mock = api_server();
    let full = volume("full", &dependencies());

    let output = run(&save, Some(&mock.address), &full, &["--key", "full-abc"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(
        &stdout,
        "Cache 'full-abc' not saved from 'target' of volume 'data': Cadence.CI skipped it: storage full",
    );
    assert!(!stdout.contains("cache: saved"), "{}", stdout);
    let state = mock.state.lock().unwrap();
    assert_eq!(
        state
            .requests
            .iter()
            .map(|(target, _)| target.as_str())
            .collect::<Vec<_>>(),
        vec![format!("POST /execution/run/{RUN_ID}/cache/save").as_str()]
    );
    drop(state);

    let _ = std::fs::remove_dir_all(&full);
}

fn misses_are_not_errors() {
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let mock = api_server();
    let empty = volume("miss", &[]);

    let output = run(
        &restore,
        Some(&mock.address),
        &empty,
        &["--keys", r#"["cargo-abc", "cargo-"]"#],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: missed");
    assert!(
        !stdout.contains("ERROR") && !stdout.contains("WARN"),
        "{}",
        stdout
    );

    let _ = std::fs::remove_dir_all(&empty);
}

fn tampered_archives_are_never_extracted() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let mock = api_server();
    let saved = volume("tampered_saved", &dependencies());

    let output = run(&save, Some(&mock.address), &saved, &["--key", "cargo-abc"]);
    assert!(output.status.success(), "{:?}", output);
    // The archive is altered in storage, its sha256 staying the one recorded.
    {
        let mut state = mock.state.lock().unwrap();
        let object = state.caches[0].object.clone();
        let content = &mut state.objects.get_mut(&object).unwrap().content;
        let middle = content.len() / 2;
        content[middle] ^= 0xff;
    }

    let restored = volume("tampered_restored", &[]);
    std::fs::remove_dir_all(restored.join("target")).unwrap();
    let output = run(
        &restore,
        Some(&mock.address),
        &restored,
        &["--keys", r#"["cargo-abc"]"#],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: missed");
    assert_contains(
        &stdout,
        "No cache restored into 'target' of volume 'data': cache 'cargo-abc' not restored, what storage gave does not match its sha256",
    );
    assert!(!restored.join("target").exists());
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
        &[
            "--key",
            "config",
            "--directory",
            leaking.join("target").to_str().unwrap(),
        ],
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
        "Cache 'config' not saved from 'target' of volume 'data': it holds the value of secret",
    );
    assert!(!stdout.contains("cache: saved"), "{}", stdout);
    let state = mock.state.lock().unwrap();
    assert!(state.caches.is_empty());
    assert!(state.objects.is_empty());
    assert_no_credentials(&output);

    let _ = std::fs::remove_dir_all(&leaking);
}

/// Requests that saved a cache, as their targets.
fn saves(mock: &Mock) -> usize {
    mock.state
        .lock()
        .unwrap()
        .requests
        .iter()
        .filter(|(target, _)| target.ends_with("/cache/save"))
        .count()
}

#[cfg(unix)]
fn steps_restore_before_and_save_after_commands() {
    let step = temp_file("run_caches", "step.mel", STEP_SCRIPT);
    let mock = api_server();
    let step_run = |volume: &Path, key: &str, exit: &str| {
        run(
            &step,
            Some(&mock.address),
            volume,
            &[
                "--key",
                key,
                "--directory",
                volume.to_str().unwrap(),
                "--exit",
                exit,
            ],
        )
    };

    // Nothing to restore: the commands build `target`, saved once they succeed, before
    // the step finishes.
    let first = volume("step_first", &[]);
    std::fs::remove_dir_all(first.join("target")).unwrap();
    let output = step_run(&first, "build-1", "0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    let saved = stdout
        .find("Cache 'build-1' saved from 'target' of volume 'data'")
        .expect(&stdout);
    assert!(
        saved < stdout.find("step: finished").expect(&stdout),
        "{}",
        stdout
    );
    assert!(!first.join("seen").exists());
    assert_eq!(saves(&mock), 1);
    assert_no_credentials(&output);

    // Restored under its key before the commands run, and not saved again.
    let second = volume("step_second", &[]);
    std::fs::remove_dir_all(second.join("target")).unwrap();
    let output = step_run(&second, "build-1", "0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(
        &stdout,
        "Cache 'build-1' restored into 'target' of volume 'data'",
    );
    assert_contains(
        &stdout,
        "Cache 'build-1' was restored under its key, not saved again",
    );
    assert!(
        second.join("seen").exists(),
        "restored before the commands ran"
    );
    assert_eq!(saves(&mock), 1);

    // Restored by prefix, and not saved when the commands fail.
    let third = volume("step_third", &[]);
    std::fs::remove_dir_all(third.join("target")).unwrap();
    let output = step_run(&third, "build-2", "1");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(
        &stdout,
        "Cache 'build-1' restored into 'target' of volume 'data'",
    );
    assert_contains(&stdout, "step: finished");
    assert!(!stdout.contains("step: success"), "{}", stdout);
    assert!(third.join("seen").exists());
    assert_eq!(saves(&mock), 1);

    for volume in [first, second, third] {
        let _ = std::fs::remove_dir_all(&volume);
    }
}

fn caches_need_cadence_ci() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let restore = temp_file("run_caches", "restore.mel", RESTORE_SCRIPT);
    let local = volume("local", &dependencies());

    let output = run(&save, None, &local, &["--key", "cargo-abc"]);
    assert_contains(
        &String::from_utf8_lossy(&output.stdout),
        "Cache 'cargo-abc' not saved from 'target' of volume 'data': caches are only saved by programs run by Cadence.CI, and MELODIUM_API_URL, MELODIUM_API_TOKEN, MELODIUM_RUN_ID are not set",
    );

    let output = run(&restore, None, &local, &["--keys", r#"["cargo-abc"]"#]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "cache: missed");
    assert_contains(
        &stdout,
        "caches are only given to programs run by Cadence.CI, and MELODIUM_API_URL, MELODIUM_API_TOKEN, MELODIUM_RUN_ID are not set",
    );

    let _ = std::fs::remove_dir_all(&local);
}

fn paths_stay_within_volumes() {
    let save = temp_file("run_caches", "save.mel", SAVE_SCRIPT);
    let mock = api_server();
    let local = volume("paths", &dependencies());

    let output = run(
        &save,
        Some(&mock.address),
        &local,
        &["--key", "outside", "--path", "../.."],
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
        "caches_are_saved_then_restored",
        caches_are_saved_then_restored,
    ));
    cases.push(("existing_keys_upload_nothing", existing_keys_upload_nothing));
    cases.push(("misses_are_not_errors", misses_are_not_errors));
    cases.push((
        "skipped_caches_upload_nothing",
        skipped_caches_upload_nothing,
    ));
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
    // The step runs `sh`.
    #[cfg(unix)]
    cases.push((
        "steps_restore_before_and_save_after_commands",
        steps_restore_before_and_save_after_commands,
    ));
    cases.push(("caches_need_cadence_ci", caches_need_cadence_ci));
    cases.push(("paths_stay_within_volumes", paths_stay_within_volumes));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
