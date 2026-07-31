use std::fs;
use std::process::{exit, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Regression test for the bug fixed in `libs/cicd-mel/mel/steps.mel`
// (`waitFinishedAndData`/`triggerDataDrained`): an orchestrator must not tear down
// a distributed connection - by calling `distrib::stop` - before the step's output
// data stream has actually finished draining, even though a separate, much faster
// `finished` signal reports "commands done" first.
//
// `stop_before_data_drained.mel` reproduces this at the protocol level: the worker
// sends a `finished` block essentially instantly, then separately streams an 8MB
// payload that takes real, measurable time to transfer over TCP. The orchestrator
// only calls `stop` once both `finished` and the fully drained `data` stream are
// observed. `received.bin` on disk must always match the full expected size - if
// `stop` fired on `finished` alone, the connection would be torn down mid-transfer
// and the file would come up short.
const EXPECTED_SIZE: u64 = 8 * 1024 * 1024;
const DIST_PORT: &str = "28041";
const RECV_KEY: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
const SEND_KEY: &str = "ffffffff-1111-2222-3333-444444444444";
const GROUP_ID: &str = "2b3c4d5e-6f78-90ab-cdef-123456789abc";
const DIST_EXIT_TIMEOUT: Duration = Duration::from_secs(90);

fn main() {
    let _ = fs::remove_file("received.bin");
    let _ = fs::remove_file("distrib_error.log");
    let _ = fs::remove_file("start_error.log");

    let mut melodium_distrib = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", GROUP_ID)
        .arg("dist")
        .arg("--localhost")
        .arg("--port")
        .arg(DIST_PORT)
        .arg("--recv-key")
        .arg(RECV_KEY)
        .arg("--send-key")
        .arg(SEND_KEY)
        .spawn()
        .expect("failed to launch Mélodium executable");

    sleep(Duration::from_millis(500));

    let mut melodium = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", GROUP_ID)
        .arg("run")
        .arg("stop_before_data_drained.mel")
        .arg("--distrib_port")
        .arg(DIST_PORT)
        .arg("--remote_key")
        .arg(RECV_KEY)
        .arg("--self_key")
        .arg(SEND_KEY)
        .spawn()
        .expect("failed to launch Mélodium executable");

    let orchestrator_status = wait_with_timeout(&mut melodium, DIST_EXIT_TIMEOUT);
    let dist_exited_naturally = wait_with_timeout(&mut melodium_distrib, DIST_EXIT_TIMEOUT);

    let _ = melodium.kill();
    let _ = melodium_distrib.kill();

    for log in ["distrib_error.log", "start_error.log"] {
        if let Ok(content) = fs::read_to_string(log) {
            if !content.trim().is_empty() {
                eprintln!("{log}: {content}");
            }
        }
    }

    match orchestrator_status {
        Some(status) if status.success() => {}
        Some(status) => {
            eprintln!("Failure, orchestrator exited with status {status}");
            exit(1);
        }
        None => {
            eprintln!("Failure, orchestrator did not exit on its own in time");
            exit(1);
        }
    }

    match dist_exited_naturally {
        Some(status) if status.success() => {}
        Some(status) => {
            eprintln!("Failure, melodium dist exited with status {status}");
            exit(1);
        }
        None => {
            eprintln!(
                "Failure, melodium dist did not exit on its own within {:?}",
                DIST_EXIT_TIMEOUT
            );
            exit(1);
        }
    }

    match fs::metadata("received.bin") {
        Ok(metadata) => {
            let size = metadata.len();
            if size != EXPECTED_SIZE {
                eprintln!(
                    "Failure, received.bin is {size} bytes, expected {EXPECTED_SIZE} \
                     (regression: connection torn down before output data fully drained)"
                );
                exit(1);
            }
            eprintln!("received.bin matches the expected {EXPECTED_SIZE} bytes");
        }
        Err(err) => {
            eprintln!("Failure, unable to read received.bin metadata: {err:?}");
            exit(1);
        }
    }
}

fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Option<std::process::ExitStatus> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    return None;
                }
                sleep(Duration::from_millis(200));
            }
            Err(_) => return None,
        }
    }
}
