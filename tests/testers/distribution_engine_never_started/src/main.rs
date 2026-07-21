use std::process::{exit, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Regression test: a `DistributionEngine` model declared in a program whose
// `start` treatment is never instantiated/triggered must not hang the whole
// engine forever. See distribution_engine_never_started.mel for the full
// explanation of the circular deadlock this used to cause.
const EXIT_TIMEOUT: Duration = Duration::from_secs(20);

fn main() {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("distribution_engine_never_started.mel")
        .spawn()
        .expect("failed to launch Mélodium executable");

    let status = wait_with_timeout(&mut melodium, EXIT_TIMEOUT);

    let _ = melodium.kill();

    match status {
        Some(status) if status.success() => {
            eprintln!("Program terminated on its own with status {status}, as expected");
        }
        Some(status) => {
            eprintln!("Failure, program exited with unexpected status {status}");
            exit(1);
        }
        None => {
            eprintln!(
                "Failure, program did not terminate on its own within {:?} (regression: \
                 DistributionEngine with a never-called `start` deadlocks the whole engine)",
                EXIT_TIMEOUT
            );
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
                sleep(Duration::from_millis(100));
            }
            Err(_) => return None,
        }
    }
}
