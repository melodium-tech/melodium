use std::process::{exit, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Regression test for the `melodium dist` container-worker not exiting on its
// own once the distributed run is actually done: it must terminate by itself,
// without being killed, shortly after the orchestrator calls `distrib::stop`.
const DIST_EXIT_TIMEOUT: Duration = Duration::from_secs(30);

fn main() {
    if std::env::var("CI").is_ok() && cfg!(target_env = "msvc") {
        // On CI for Windows MSVC, for now, we skip tests involving TLS because schannel is not able to register custom certificates.
        exit(0);
    }

    let _ = std::fs::remove_file("ok.log");
    let _ = std::fs::remove_file("failure.log");
    let _ = std::fs::remove_file("reply.log");

    let mut melodium_distrib = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", "6f2d9a2a-4b0a-4a24-9f1a-3f7f6a6c4d31")
        .arg("dist")
        .arg("--localhost")
        .arg("--port")
        .arg("28016")
        .arg("--recv-key")
        .arg("2f6e2a2e-6b4b-5a2e-9b3e-2f6a2e6b4b5a")
        .arg("--send-key")
        .arg("5a2e6b4b-2f6e-2a2e-9b3e-6b4b5a2e2f6a")
        .spawn()
        .expect("failed to launch Mélodium executable");

    let mut melodium = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", "6f2d9a2a-4b0a-4a24-9f1a-3f7f6a6c4d31")
        .arg("run")
        .arg("process_distributed.mel")
        .arg("--distrib_port")
        .arg("28016")
        .arg("--remote_key")
        .arg("2f6e2a2e-6b4b-5a2e-9b3e-2f6a2e6b4b5a")
        .arg("--self_key")
        .arg("5a2e6b4b-2f6e-2a2e-9b3e-6b4b5a2e2f6a")
        .spawn()
        .expect("failed to launch Mélodium executable");

    // Give the orchestrator time to connect, distribute the work, receive the
    // reply, and call `distrib::stop` on its own; it is not killed here.
    let orchestrator_status = wait_with_timeout(&mut melodium, Duration::from_secs(30));

    // The real assertion: once the orchestrator is done and has told the
    // remote side to stop, `melodium dist` must exit by itself as well,
    // without ever being killed. If the watchdog added in
    // melodium-distribution/src/listen.rs regresses, this call times out
    // with the worker still running.
    let dist_exited_naturally = wait_with_timeout(&mut melodium_distrib, DIST_EXIT_TIMEOUT);

    // Only reached if either process is still alive; failure path cleanup.
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
                "Failure, melodium dist did not exit on its own within {:?} after the run finished \
                 (worker likely stuck waiting on distribution end)",
                DIST_EXIT_TIMEOUT
            );
            exit(1);
        }
    }

    match std::fs::read_to_string("reply.log") {
        Ok(reply) => {
            if reply != "Pingouin" {
                eprintln!("Failure, unexpected reply: {reply}");
                exit(1);
            }
        }
        Err(err) => {
            eprintln!("Failure, unable to read reply.log: {err:?}");
            exit(1);
        }
    }
}

/// Polls the child for natural exit, without ever killing it, up to `timeout`.
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
