use std::process::{exit, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Regression test for the bug where `melodium-distribution/src/listen.rs`'s teardown
// watchdog raced the ENTIRE `limit`/`live`/`run`/`logs`/`debug` join, instead of only
// the `logs`/`debug` tail after real work already completed. That made the watchdog a
// fixed deadline on the whole job rather than a safety net for a genuinely stalled
// connection: any distributed run taking longer than `teardown_timeout()` (60s default)
// had its connection torn down mid-job, even though the worker process itself kept
// running its commands to completion, orphaned and disconnected.
//
// `process_distributed_slow.mel`'s worker deliberately sleeps 8 seconds - longer than
// the shortened `MELODIUM_DIST_TEARDOWN_TIMEOUT_SECS` this harness configures - before
// replying. The run must still complete and produce the expected reply; a regression
// here manifests as the connection being cut and `reply.log` never being written.
const DIST_PORT: &str = "28053";
const RECV_KEY: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
const SEND_KEY: &str = "ffffffff-1111-2222-3333-444444444444";
const GROUP_ID: &str = "4b5c6d7e-8f90-4bcd-8ef1-23456789abcd";
// Shorter than the worker's 8s sleep: proves the connection survives past this window
// as long as real work is progressing, rather than being torn down on a fixed schedule.
const TEARDOWN_TIMEOUT_SECS: u64 = 3;
const RUN_TIMEOUT: Duration = Duration::from_secs(30);

fn main() {
    let _ = std::fs::remove_file("failure.log");
    let _ = std::fs::remove_file("reply.log");
    let _ = std::fs::remove_file("ok.log");

    let mut melodium_distrib = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", GROUP_ID)
        .env(
            "MELODIUM_DIST_TEARDOWN_TIMEOUT_SECS",
            TEARDOWN_TIMEOUT_SECS.to_string(),
        )
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
        .arg("process_distributed_slow.mel")
        .arg("--distrib_port")
        .arg(DIST_PORT)
        .arg("--remote_key")
        .arg(RECV_KEY)
        .arg("--self_key")
        .arg(SEND_KEY)
        .spawn()
        .expect("failed to launch Mélodium executable");

    let orchestrator_status = wait_with_timeout(&mut melodium, RUN_TIMEOUT);
    let dist_exited_naturally = wait_with_timeout(&mut melodium_distrib, RUN_TIMEOUT);

    let _ = melodium.kill();
    let _ = melodium_distrib.kill();

    eprintln!(
        "failure.log: {}",
        std::fs::read_to_string("failure.log").unwrap_or("Rien".to_string())
    );
    eprintln!(
        "reply.log: {}",
        std::fs::read_to_string("reply.log").unwrap_or("Rien".to_string())
    );

    match orchestrator_status {
        Some(status) if status.success() => {}
        Some(status) => {
            eprintln!(
                "Failure, orchestrator exited with status {status} (regression: teardown \
                 watchdog likely cut the connection before the slow-but-healthy job finished)"
            );
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
            eprintln!("Failure, melodium dist did not exit on its own in time");
            exit(1);
        }
    }

    match std::fs::read_to_string("reply.log") {
        Ok(reply) => {
            if reply != "Pingouin" {
                eprintln!(
                    "Failure, unexpected reply: {reply} (regression: connection torn down \
                     before the 8s worker-side sleep completed)"
                );
                exit(1);
            }
        }
        Err(err) => {
            eprintln!(
                "Failure, unable to read reply.log: {err:?} (regression: teardown watchdog \
                 fired at {TEARDOWN_TIMEOUT_SECS}s while the worker was still genuinely \
                 running its 8s command)"
            );
            exit(1);
        }
    }

    eprintln!("process_distributed_slow passed");
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
