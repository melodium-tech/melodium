use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{exit, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Regression test for `melodium dist` waiting forever when it never receives
// `Message::Ended` from the distant orchestrator, because the connection goes
// silently unresponsive (no more bytes in either direction, but not cleanly
// closed either - e.g. a dropped link, a black-holing firewall). A cleanly
// closed socket already makes `recv_message` fail immediately and `run` was
// already able to unblock from that before any fix; the case this guards is
// the connection staying open but silent, which previously left `dist`
// waiting forever.
//
// A man-in-the-middle proxy sits between the orchestrator and the real
// `melodium dist` listener, forwards traffic transparently in both
// directions, but only lets a limited number of writes through on the
// orchestrator -> dist direction (enough to cover the handshake and
// `Message::LoadAndLaunch`, which is what lets the engine genuinely launch
// and reach the post-launch teardown code the watchdog lives in), then drops
// everything from the orchestrator afterward without ever closing either
// socket - so `Message::Ended` (sent later, once `distrib::stop` runs) never
// arrives. `dist` must then still exit on its own within its (shortened, via
// env var) grace period, unless the watchdog added in
// melodium-distribution/src/listen.rs regresses. Counting writes rather than
// racing on wall-clock time keeps this deterministic regardless of how fast
// the local loopback exchange happens to run.

const DIST_PORT: u16 = 28017;
const PROXY_PORT: u16 = 28018;
// Number of orchestrator -> dist writes let through before going silent on
// that direction: AskDistribution, then LoadAndLaunch (observed as a single
// larger write in practice, but allow a little slack).
const ORCHESTRATOR_WRITES_ALLOWED: usize = 6;
const TEARDOWN_TIMEOUT_SECS: u64 = 3;
const DIST_EXIT_TIMEOUT: Duration = Duration::from_secs(30);

fn main() {
    if std::env::var("CI").is_ok() && cfg!(target_env = "msvc") {
        exit(0);
    }

    let _ = std::fs::remove_file("failure.log");
    let _ = std::fs::remove_file("reply.log");
    let _ = std::fs::remove_file("ok.log");

    let mut melodium_distrib = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", "8b1d2e3a-9c4f-4a11-8b2d-1e3a9c4f4a11")
        .env(
            "MELODIUM_DIST_TEARDOWN_TIMEOUT_SECS",
            TEARDOWN_TIMEOUT_SECS.to_string(),
        )
        .env(
            "MELODIUM_DIST_MONITORING_TIMEOUT_SECS",
            TEARDOWN_TIMEOUT_SECS.to_string(),
        )
        .arg("dist")
        .arg("--localhost")
        .arg("--port")
        .arg(DIST_PORT.to_string())
        .arg("--recv-key")
        .arg("3c1e2a2e-6b4b-5a2e-9b3e-2f6a2e6b4b5a")
        .arg("--send-key")
        .arg("7a2e6b4b-3c1e-2a2e-9b3e-6b4b5a2e2f6a")
        .spawn()
        .expect("failed to launch Mélodium executable");

    // Give `melodium dist` time to bind before the proxy dials it.
    sleep(Duration::from_millis(500));

    let proxy = std::thread::spawn(run_proxy);

    let mut melodium = Command::new("melodium")
        .env("MELODIUM_GROUP_ID", "8b1d2e3a-9c4f-4a11-8b2d-1e3a9c4f4a11")
        .arg("run")
        .arg("process_distributed.mel")
        .arg("--distrib_port")
        .arg(PROXY_PORT.to_string())
        .arg("--remote_key")
        .arg("3c1e2a2e-6b4b-5a2e-9b3e-2f6a2e6b4b5a")
        .arg("--self_key")
        .arg("7a2e6b4b-3c1e-2a2e-9b3e-6b4b5a2e2f6a")
        .spawn()
        .expect("failed to launch Mélodium executable");

    // The real assertion: `melodium dist` must exit on its own within the
    // shortened watchdog window, even though it never receives Message::Ended
    // and its socket is never closed, just gone silent via the proxy.
    let dist_exited_naturally = wait_with_timeout(&mut melodium_distrib, DIST_EXIT_TIMEOUT);

    // Cleanup: the orchestrator can't ever get its reply once the proxy goes
    // silent, so it is killed here; only `melodium dist`'s exit is under test.
    let _ = melodium.kill();
    let _ = melodium_distrib.kill();
    drop(proxy); // proxy threads are daemon-like; process exit reclaims them

    match dist_exited_naturally {
        Some(status) if status.success() => {
            eprintln!("melodium dist exited on its own with status {status}, as expected");
        }
        Some(status) => {
            eprintln!("Failure, melodium dist exited with unexpected status {status}");
            exit(1);
        }
        None => {
            eprintln!(
                "Failure, melodium dist did not exit on its own within {:?} after its connection \
                 went silent (watchdog regression: worker stuck waiting on distribution end)",
                DIST_EXIT_TIMEOUT
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
                sleep(Duration::from_millis(200));
            }
            Err(_) => return None,
        }
    }
}

/// Accepts a single connection from the orchestrator, relays it to the real
/// `melodium dist` listener. The dist -> orchestrator direction is always
/// forwarded (so the engine genuinely launches and the orchestrator gets its
/// reply); the orchestrator -> dist direction stops being forwarded after
/// `ORCHESTRATOR_WRITES_ALLOWED` writes, without ever closing either socket,
/// so `Message::Ended` (sent afterward) never reaches `dist`.
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

    let orchestrator_to_dist =
        std::thread::spawn(move || pump_limited(client_to_server, server));
    let dist_to_orchestrator =
        std::thread::spawn(move || pump_unlimited(server_to_client, client));

    // Keep both streams alive (never dropped) for the lifetime of the
    // process, even after both pump threads have gone silent, so the sockets
    // stay open but unresponsive rather than being closed.
    let _ = orchestrator_to_dist.join();
    let _ = dist_to_orchestrator.join();
    loop {
        sleep(Duration::from_secs(3600));
    }
}

/// Forwards up to `ORCHESTRATOR_WRITES_ALLOWED` reads worth of bytes, then
/// keeps draining the socket (to avoid backpressure making the peer see an
/// immediate error) but never writes again and never closes the connection.
fn pump_limited(mut from: TcpStream, mut to: TcpStream) {
    let mut buffer = [0u8; 4096];
    let mut writes = 0usize;
    loop {
        let _ = from.set_read_timeout(Some(Duration::from_millis(200)));
        match from.read(&mut buffer) {
            Ok(0) => return,
            Ok(n) => {
                if writes < ORCHESTRATOR_WRITES_ALLOWED {
                    if to.write_all(&buffer[..n]).is_err() {
                        return;
                    }
                    writes += 1;
                }
                // else: silently drop, but keep the connection itself open.
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

/// Forwards everything, unconditionally, until the connection closes.
fn pump_unlimited(mut from: TcpStream, mut to: TcpStream) {
    let mut buffer = [0u8; 4096];
    loop {
        let _ = from.set_read_timeout(Some(Duration::from_millis(200)));
        match from.read(&mut buffer) {
            Ok(0) => return,
            Ok(n) => {
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
