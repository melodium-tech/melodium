use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{exit, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Regression test for the `protocol.rs` `TIMEOUT`/probe-interval bug: both sides send
// a `Message::Probe` every 10s to keep the connection's read timeout from firing on an
// otherwise-idle-but-healthy link, but if that internal timeout is too close to the
// probe interval, ordinary network/scheduling delay (e.g. several concurrent
// distributed connections competing for CPU under a real CI load) can make a message
// arrive just late enough to trip the timeout and tear down a perfectly healthy
// connection. This test proxies every byte between orchestrator and worker through a
// relay that delays each forwarded chunk by `HOP_DELAY`, which is longer than the 10s
// probe interval but still comfortably under the configured protocol timeout - the
// run must still complete successfully despite that latency, proving the timeout
// isn't so tight that ordinary delay looks like a dead connection.
const DIST_PORT: u16 = 28020;
const PROXY_PORT: u16 = 28021;
// Longer than the 10s probe interval (so it genuinely exercises the same margin
// concurrent CI load would eat into), comfortably under PROTOCOL_TIMEOUT_SECS.
const HOP_DELAY: Duration = Duration::from_secs(3);
// Shortened via env var so the test doesn't need to wait out the real 60s production
// default; still comfortably longer than HOP_DELAY on each single hop.
const PROTOCOL_TIMEOUT_SECS: u64 = 20;
const RUN_TIMEOUT: Duration = Duration::from_secs(120);

fn main() {
    if std::env::var("CI").is_ok() && cfg!(target_env = "msvc") {
        exit(0);
    }

    let _ = std::fs::remove_file("failure.log");
    let _ = std::fs::remove_file("reply.log");
    let _ = std::fs::remove_file("ok.log");

    let mut melodium_distrib = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", "5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e8f")
        .env(
            "MELODIUM_DIST_PROTOCOL_TIMEOUT_SECS",
            PROTOCOL_TIMEOUT_SECS.to_string(),
        )
        .arg("dist")
        .arg("--localhost")
        .arg("--port")
        .arg(DIST_PORT.to_string())
        .arg("--recv-key")
        .arg("6d7e8f9a-0b4c-1d8e-2f3a-4b5c6d7e8f9a")
        .arg("--send-key")
        .arg("7e8f9a0b-4c1d-8e2f-3a4b-5c6d7e8f9a0b")
        .spawn()
        .expect("failed to launch Mélodium executable");

    sleep(Duration::from_millis(500));

    let proxy = std::thread::spawn(run_proxy);
    sleep(Duration::from_millis(200));

    let mut melodium = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", "5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e8f")
        .env(
            "MELODIUM_DIST_PROTOCOL_TIMEOUT_SECS",
            PROTOCOL_TIMEOUT_SECS.to_string(),
        )
        .arg("run")
        .arg("process_distributed.mel")
        .arg("--distrib_port")
        .arg(PROXY_PORT.to_string())
        .arg("--remote_key")
        .arg("6d7e8f9a-0b4c-1d8e-2f3a-4b5c6d7e8f9a")
        .arg("--self_key")
        .arg("7e8f9a0b-4c1d-8e2f-3a4b-5c6d7e8f9a0b")
        .spawn()
        .expect("failed to launch Mélodium executable");

    let orchestrator_status = wait_with_timeout(&mut melodium, RUN_TIMEOUT);
    let dist_exited_naturally = wait_with_timeout(&mut melodium_distrib, RUN_TIMEOUT);

    let _ = melodium.kill();
    let _ = melodium_distrib.kill();
    drop(proxy);

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
                "Failure, orchestrator exited with status {status} (regression: delayed-but- \
                 healthy connection was treated as dead)"
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
                eprintln!("Failure, unexpected reply: {reply}");
                exit(1);
            }
        }
        Err(err) => {
            eprintln!("Failure, unable to read reply.log: {err:?}");
            exit(1);
        }
    }

    eprintln!("protocol_timeout_survives_jitter passed");
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

/// Accepts a single connection from the orchestrator, relays it to the real
/// `melodium dist` listener, delaying every forwarded chunk by `HOP_DELAY` in
/// both directions - simulating scheduling/network jitter without ever going
/// silent or dropping data.
fn run_proxy() {
    let listener = TcpListener::bind(("127.0.0.1", PROXY_PORT)).expect("failed to bind proxy");
    let (client, _) = match listener.accept() {
        Ok(accepted) => accepted,
        Err(_) => return,
    };
    let server = match TcpStream::connect(("127.0.0.1", DIST_PORT)) {
        Ok(stream) => stream,
        Err(_) => return,
    };

    let client_to_server = client.try_clone().expect("clone failed");
    let server_to_client = server.try_clone().expect("clone failed");

    let orchestrator_to_dist = std::thread::spawn(move || pump_delayed(client_to_server, server));
    let dist_to_orchestrator =
        std::thread::spawn(move || pump_delayed(server_to_client, client));

    let _ = orchestrator_to_dist.join();
    let _ = dist_to_orchestrator.join();
}

/// Forwards everything, delaying each read chunk by `HOP_DELAY` before writing
/// it onward, until the connection closes.
fn pump_delayed(mut from: TcpStream, mut to: TcpStream) {
    let mut buffer = [0u8; 4096];
    loop {
        let _ = from.set_read_timeout(Some(Duration::from_millis(200)));
        match from.read(&mut buffer) {
            Ok(0) => return,
            Ok(n) => {
                sleep(HOP_DELAY);
                if to.write_all(&buffer[..n]).is_err() {
                    return;
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut =>
            {
                continue
            }
            Err(_) => return,
        }
    }
}
