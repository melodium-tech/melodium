//! Secret values never appear in the outputs of a run: standard output and error, the
//! `--logs` file, the `--debug` file at detailed level, and what `--api-report` sends
//! (logs, debug events, program dump and details), nor in `.jeu` packages and
//! documentation. A value read by the program, concealed, then plainly revealed
//! through `std/secret::reveal` is ordinary data, so it only appears in debug data
//! events; secrets from the environment and from files are never plainly revealed.
//!
//! The API and its uploads are a mock answering every request.

#![cfg_attr(not(unix), allow(dead_code, unused_imports))]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

const SCRIPT: &str = include_str!("../../../secret_leaks.mel");

const HIDDEN: &str = "hidden-leak-sentinel";
const SHOWN: &str = "shown-leak-sentinel";
const API_TOKEN: &str = "api-token-leak-sentinel";

fn directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_secret_leaks_{}_{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn standalone_script(directory: &Path) -> PathBuf {
    let script = directory.join("secret_leaks.mel");
    std::fs::write(&script, SCRIPT).unwrap();
    script
}

/// Request received by the mock API: method and path, then body.
struct Received {
    target: String,
    body: String,
}

/// Starts a mock of the API and of the storage it gives for uploads, giving its address
/// and the requests it receives.
fn api_server() -> (String, Arc<Mutex<Vec<Received>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let received = Arc::new(Mutex::new(Vec::new()));

    let requests = Arc::clone(&received);
    let uploads = address.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let requests = Arc::clone(&requests);
                let uploads = uploads.clone();
                std::thread::spawn(move || {
                    let _ = answer(stream, &uploads, &requests);
                });
            }
        }
    });

    (address, received)
}

fn answer(
    mut stream: TcpStream,
    uploads: &str,
    requests: &Mutex<Vec<Received>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_lowercase(), value.trim().to_string());
        }
    }

    let mut body = Vec::new();
    if headers.get("transfer-encoding").map(|value| value.as_str()) == Some("chunked") {
        loop {
            let mut size = String::new();
            reader.read_line(&mut size)?;
            let size = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
            let mut chunk = vec![0u8; size + 2];
            reader.read_exact(&mut chunk)?;
            if size == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..size]);
        }
    } else if let Some(length) = headers.get("content-length") {
        body.resize(length.parse().unwrap_or(0), 0);
        reader.read_exact(&mut body)?;
    }
    let body = String::from_utf8_lossy(&body).to_string();

    let mut parts = request_line.split_whitespace();
    let target = format!(
        "{} {}",
        parts.next().unwrap_or_default(),
        parts.next().unwrap_or_default()
    );

    let response = if target.ends_with("/execution/report/request") {
        // Gives back the run and group identifiers, with uploads to this server.
        format!(
            r#"{},"dashboard":null,"logs":{{"presigned_post_s3":{{"uri":"{uploads}/storage","fields":{{}},"path":"logs"}}}},"debug":{{"presigned_post_s3":{{"uri":"{uploads}/storage","fields":{{}},"path":"debug"}}}},"program":{{"presigned_put_s3":{{"uri":"{uploads}/storage/program","headers":{{}}}}}}}}"#,
            body.trim().trim_end_matches('}')
        )
    } else {
        "{}".to_string()
    };

    requests.lock().unwrap().push(Received { target, body });

    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
        response.len()
    )
}

#[cfg(unix)]
fn secret_values_never_appear_in_run_outputs() {
    let directory = directory("run");
    let script = standalone_script(&directory);
    let logs = directory.join("logs");
    let debug = directory.join("debug");
    let (address, requests) = api_server();
    std::fs::write(directory.join("shown"), SHOWN).unwrap();

    let output = Command::new("melodium")
        .args(["run", "--api-report", "--debug-level", "detailed", "--logs"])
        .arg(&logs)
        .arg("--debug")
        .arg(&debug)
        .arg(&script)
        .arg("--directory")
        .arg(&directory)
        .env("MELODIUM_SECRET_LEAKS_HIDDEN", HIDDEN)
        .env("MELODIUM_API_URL", &address)
        .env("MELODIUM_API_TOKEN", API_TOKEN)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{}{}", stdout, stderr);

    // The command got the never revealed secret.
    assert_eq!(
        std::fs::read_to_string(directory.join("hidden")).unwrap(),
        HIDDEN
    );

    let logs = std::fs::read_to_string(&logs).unwrap();
    let debug = std::fs::read_to_string(&debug).unwrap();
    // The shown value was revealed: masked in logs, sent as data.
    assert!(logs.contains("shown: <secret \"shown\">"), "{}", logs);
    assert!(debug.contains(SHOWN));

    let requests = requests.lock().unwrap();
    let targets: Vec<&str> = requests
        .iter()
        .map(|request| request.target.as_str())
        .collect();
    for expected in [
        "POST /execution/run/start",
        "POST /execution/report/request",
        "PUT /storage/program",
    ] {
        assert!(targets.contains(&expected), "{:?}", targets);
    }
    assert!(targets
        .iter()
        .any(|target| target.ends_with("/program/details")));
    let debug_uploads: Vec<&Received> = requests
        .iter()
        .filter(|request| request.body.contains("debug/debug_"))
        .collect();
    let other_requests: Vec<&Received> = requests
        .iter()
        .filter(|request| !request.body.contains("debug/debug_"))
        .collect();
    assert!(!debug_uploads.is_empty(), "{:?}", targets);
    assert!(other_requests
        .iter()
        .any(|request| request.body.contains("logs/logs_")));

    for value in [HIDDEN, API_TOKEN] {
        assert!(!stdout.contains(value));
        assert!(!stderr.contains(value));
        assert!(!logs.contains(value), "{}", logs);
        assert!(!debug.contains(value));
        for request in requests.iter() {
            assert!(!request.body.contains(value), "{}", request.target);
        }
    }

    // Plainly revealed: only in data events.
    assert!(!stdout.contains(SHOWN));
    assert!(!stderr.contains(SHOWN));
    assert!(!logs.contains(SHOWN), "{}", logs);
    for request in other_requests {
        assert!(!request.body.contains(SHOWN), "{}", request.target);
    }

    let _ = std::fs::remove_dir_all(&directory);
}

fn secret_values_never_appear_in_packages_nor_documentation() {
    let directory = directory("package");
    let package = directory.join("secret_leaks");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("Compo.toml"),
        "name = \"secret_leaks\"\nversion = \"0.1.0\"\n\n[dependencies]\nstd = \"0.11.0\"\nprocess = \"0.11.0\"\nfs = \"0.11.0\"\n\n[entrypoints]\nmain = \"secret_leaks::main\"\n",
    )
    .unwrap();
    // The same program without the header of standalone files.
    let lib_root = SCRIPT
        .lines()
        .filter(|line| !line.starts_with("#!"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(package.join("lib-root.mel"), lib_root).unwrap();
    let script = standalone_script(&directory);

    let jeu = directory.join("secret_leaks.jeu");
    let output = Command::new("melodium")
        .args(["jeu", "build"])
        .arg(&package)
        .arg(&jeu)
        .env("MELODIUM_SECRET_LEAKS_HIDDEN", HIDDEN)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let documentation = directory.join("documentation");
    let output = Command::new("melodium")
        .args(["doc", "--file"])
        .arg(&script)
        .arg(&documentation)
        .env("MELODIUM_SECRET_LEAKS_HIDDEN", HIDDEN)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut files = vec![jeu];
    let mut directories = vec![documentation];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                directories.push(path);
            } else {
                files.push(path);
            }
        }
    }
    assert!(files.len() > 1);
    let mut documented = false;
    for file in files {
        let content = std::fs::read(&file).unwrap();
        let content = String::from_utf8_lossy(&content);
        documented |= content.contains("MELODIUM_SECRET_LEAKS_HIDDEN");
        for value in [HIDDEN, SHOWN] {
            assert!(!content.contains(value), "{}", file.display());
        }
    }
    // The default of the secret parameter is documented as its locator.
    assert!(documented);

    let _ = std::fs::remove_dir_all(&directory);
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    // The program runs `sh`.
    #[cfg(unix)]
    cases.push((
        "secret_values_never_appear_in_run_outputs",
        secret_values_never_appear_in_run_outputs,
    ));
    cases.push((
        "secret_values_never_appear_in_packages_nor_documentation",
        secret_values_never_appear_in_packages_nor_documentation,
    ));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
