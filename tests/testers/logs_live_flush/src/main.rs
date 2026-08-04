use std::fs;
use std::process::{exit, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Regression test: `write_logs`/`write_debug` (melodium/src/lib.rs) used to only
// flush their `BufWriter` once, right when the whole run ended, instead of after
// each line/event. A caller tailing the log or debug file live during a long run
// (e.g. `tail -f`) would see nothing on disk until the process exited, even though
// content had already been produced minutes earlier.
//
// `logs_live_flush.mel` logs one line, then spends a few real seconds inside a
// spawned `sleep` command before logging a second line and exiting. This harness
// starts the program, waits partway through that sleep (well before the process
// can have exited), and asserts the first line is already present on disk in both
// the `--logs` and `--debug` files - proving each line/event is flushed as it is
// written, not batched until exit.

const MID_RUN_CHECK_DELAY: Duration = Duration::from_secs(2);
const PROCESS_EXIT_TIMEOUT: Duration = Duration::from_secs(20);

fn main() {
    let _ = fs::remove_file("logs_live_flush.logs");
    let _ = fs::remove_file("logs_live_flush.debug");

    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("logs_live_flush.mel")
        .arg("--logs")
        .arg("logs_live_flush.logs")
        .arg("--debug")
        .arg("logs_live_flush.debug")
        .spawn()
        .expect("failed to launch Mélodium executable");

    // The program sleeps ~4s between its two log lines; checking partway through
    // that sleep proves the first line reached disk well before the process
    // could possibly have exited (and therefore before any "flush on drop/exit"
    // fallback could explain it).
    sleep(MID_RUN_CHECK_DELAY);

    let mid_run_logs = fs::read_to_string("logs_live_flush.logs").unwrap_or_default();
    let mid_run_debug = fs::read_to_string("logs_live_flush.debug").unwrap_or_default();

    let status = wait_with_timeout(&mut melodium, PROCESS_EXIT_TIMEOUT);
    let _ = melodium.kill();

    let mut failed = false;

    if status.is_none() {
        eprintln!(
            "Failure, melodium run did not exit on its own within {:?}",
            PROCESS_EXIT_TIMEOUT
        );
        failed = true;
    }

    if !mid_run_logs.contains("first-line-written") {
        eprintln!(
            "Failure, logs file did not contain the first line while the process was \
             still running (regression: logs only flushed at exit). Content at check time: {:?}",
            mid_run_logs
        );
        failed = true;
    } else {
        eprintln!("logs file correctly contained the first line mid-run");
    }

    if mid_run_logs.contains("second-line-written") {
        eprintln!(
            "Warning: second line was already present at the mid-run check; the test's \
             timing margin may be too tight to be a meaningful assertion."
        );
    }

    if !mid_run_debug.contains("first-line-written") {
        eprintln!(
            "Failure, debug file did not contain the first line's event while the process \
             was still running (regression: debug only flushed at exit). Content length at \
             check time: {} bytes",
            mid_run_debug.len()
        );
        failed = true;
    } else {
        eprintln!("debug file correctly contained the first line's event mid-run");
    }

    let final_logs = fs::read_to_string("logs_live_flush.logs").unwrap_or_default();
    if !final_logs.contains("second-line-written") {
        eprintln!(
            "Failure, final logs file did not contain the second line after completion: {:?}",
            final_logs
        );
        failed = true;
    }

    if failed {
        exit(1);
    }

    eprintln!("logs_live_flush passed");
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
