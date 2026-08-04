use std::fs;
use std::process::{exit, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Regression test for the bug where `logs`/`debug` forwarding from a distributed
// worker back to the orchestrator (`melodium-distribution/src/listen.rs`) was only
// polled once `limit`/`live`/`run` (the run's real work) already completed, instead
// of concurrently with it - because those two futures were only referenced inside a
// second, later `join!`, and an unpolled future does nothing. That silently delayed
// every log/debug event until the whole job was already over, even though the worker
// was also, separately and correctly, still reporting them live to the API the whole
// time - which is what made this so easy to miss.
//
// `process_distributed_logs_live.mel`'s worker logs a distinctive message
// immediately, then spends 6 seconds on an unrelated, healthy, slow command before
// replying. The orchestrator's `--logs` file must contain that message while the
// worker is still in that slow phase, not only once the whole run has finished.
const DIST_PORT: &str = "28063";
const RECV_KEY: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
const SEND_KEY: &str = "ffffffff-1111-2222-3333-444444444444";
const GROUP_ID: &str = "7e8f90ab-cdef-4123-8456-789abcdef012";
// The worker sleeps 6s before replying; checking well before that proves the log
// arrived while it was still in that sleep, not only after the run completed.
const MID_RUN_CHECK_DELAY: Duration = Duration::from_secs(3);
const RUN_TIMEOUT: Duration = Duration::from_secs(30);

fn main() {
    let _ = fs::remove_file("failure.log");
    let _ = fs::remove_file("reply.log");
    let _ = fs::remove_file("ok.log");
    let _ = fs::remove_file("orchestrator.logs");

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
        .arg("--logs")
        .arg("orchestrator.logs")
        .arg("process_distributed_logs_live.mel")
        .arg("--distrib_port")
        .arg(DIST_PORT)
        .arg("--remote_key")
        .arg(RECV_KEY)
        .arg("--self_key")
        .arg(SEND_KEY)
        .spawn()
        .expect("failed to launch Mélodium executable");

    sleep(MID_RUN_CHECK_DELAY);
    let mid_run_logs = fs::read_to_string("orchestrator.logs").unwrap_or_default();

    let orchestrator_status = wait_with_timeout(&mut melodium, RUN_TIMEOUT);
    let dist_exited_naturally = wait_with_timeout(&mut melodium_distrib, RUN_TIMEOUT);

    let _ = melodium.kill();
    let _ = melodium_distrib.kill();

    eprintln!(
        "failure.log: {}",
        fs::read_to_string("failure.log").unwrap_or("Rien".to_string())
    );
    eprintln!(
        "reply.log: {}",
        fs::read_to_string("reply.log").unwrap_or("Rien".to_string())
    );

    let mut failed = false;

    if !mid_run_logs.contains("worker-received-request") {
        eprintln!(
            "Failure, orchestrator.logs did not contain the worker's log line while the worker \
             was still running (regression: logs/debug forwarding only polled after the job \
             already finished). Content at check time: {:?}",
            mid_run_logs
        );
        failed = true;
    } else {
        eprintln!("orchestrator.logs correctly contained the worker's log line mid-run");
    }

    match orchestrator_status {
        Some(status) if status.success() => {}
        Some(status) => {
            eprintln!("Failure, orchestrator exited with status {status}");
            failed = true;
        }
        None => {
            eprintln!("Failure, orchestrator did not exit on its own in time");
            failed = true;
        }
    }

    match dist_exited_naturally {
        Some(status) if status.success() => {}
        Some(status) => {
            eprintln!("Failure, melodium dist exited with status {status}");
            failed = true;
        }
        None => {
            eprintln!("Failure, melodium dist did not exit on its own in time");
            failed = true;
        }
    }

    match fs::read_to_string("reply.log") {
        Ok(reply) => {
            if reply != "Pingouin" {
                eprintln!("Failure, unexpected reply: {reply}");
                failed = true;
            }
        }
        Err(err) => {
            eprintln!("Failure, unable to read reply.log: {err:?}");
            failed = true;
        }
    }

    if failed {
        exit(1);
    }

    eprintln!("process_distributed_logs_live passed");
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
